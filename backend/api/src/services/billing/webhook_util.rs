//! Shared primitives for webhook signature verification.
//!
//! Every billing provider reads its signature header(s) from the incoming
//! request and authenticates the raw body. Previously each provider carried its
//! own hand-rolled `constant_time_eq_str` (9 copies) and recomputed HMAC-SHA256
//! inline; this module centralises the constant-time comparison, the HMAC
//! computation, and the Ed25519 verification Paddle needs. See plan Phase 1c.

use axum::http::HeaderMap;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Maximum tolerated clock skew (in seconds) between the provider's timestamp
/// and our wall clock. Events outside this window are rejected as replays
/// (CWE-294 / OWASP A07). Five minutes is the conventional budget (Stripe's own
/// default).
pub const MAX_SKEW_SECS: i64 = 5 * 60;

type HmacSha256 = Hmac<Sha256>;

/// Compute `HMAC-SHA256(key, msg)` and return the lowercase hex digest.
pub fn hmac_sha256_hex(key: &[u8], msg: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(msg);
    hex::encode(mac.finalize().into_bytes())
}

/// Verify a `HMAC-SHA256(key, msg)` tag provided as hex against the recomputed
/// digest, in constant time. Fails (returns `false`) if the provided tag is not
/// valid hex or does not match.
pub fn verify_hmac_sha256_hex(key: &[u8], msg: &[u8], provided_hex: &str) -> bool {
    let expected = hmac_sha256_hex(key, msg);
    ct_eq(expected.as_bytes(), provided_hex.trim().as_bytes())
}

/// Constant-time byte comparison. A length mismatch returns `false` immediately
/// (the length of a MAC tag / hex digest is fixed and public, so leaking it is
/// not a timing oracle). For equal lengths the comparison is constant-time.
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}

/// Read a header value as an owned `String`, or `None` if absent / non-ASCII.
pub fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Reject a timestamp that is too far from `now_secs`. `ts_secs` is the
/// provider-supplied timestamp in seconds since the Unix epoch; `now_secs` is
/// the server's current time the same way. Returns `true` if within budget.
pub fn timestamp_fresh(ts_secs: i64, now_secs: i64) -> bool {
    ts_secs.saturating_sub(now_secs).abs() <= MAX_SKEW_SECS
}

/// Decode a Polar / Standard Webhooks secret into the raw HMAC key.
///
/// Polar webhook secrets are issued as `whsec_<base64>`; the `whsec_` prefix is
/// the textual marker and the base64 portion decodes to the 32-byte signing key
/// (Standard Webhooks spec, standardwebhooks.com). We strip a leading `whsec_`
/// when present, then try to base64-decode the remainder. If decoding fails —
/// e.g. the stored value is already a raw 32-byte key, or a non-canonical
/// string was configured — we fall back to the raw bytes so a misconfigured
/// secret fails at the signature check rather than at key derivation. The
/// base64 path is preferred because that is how Polar ships the secret.
pub fn standard_webhooks_key(secret: &str) -> Vec<u8> {
    use base64::Engine;
    let trimmed = secret.strip_prefix("whsec_").unwrap_or(secret);
    match base64::engine::general_purpose::STANDARD.decode(trimmed) {
        Ok(key) => key,
        // Not valid base64 — use the raw bytes as-is (raw 32-byte key path).
        Err(_) => trimmed.as_bytes().to_vec(),
    }
}

/// Verify a Standard Webhooks (polar.sh / standardwebhooks.com) signature.
///
/// The signed message is `"{webhook_id}.{webhook_timestamp}.{body}"`, signed
/// with HMAC-SHA256 using the raw key derived from the `whsec_<base64>` secret,
/// and transmitted base64-encoded. The `webhook-signature` header may carry
/// multiple whitespace-separated `v1,<base64>` entries (key rotation); we accept
/// if ANY entry matches the recomputed digest, compared in constant time. The
/// `webhook-timestamp` header (unix seconds) is bound into the signed message
/// AND independently checked against a 5-minute replay window.
///
/// Returns `true` only when all three headers are present, the timestamp is
/// fresh, and at least one `v1` signature entry matches.
pub fn verify_standard_webhooks(
    headers: &HeaderMap,
    secret: &str,
    body: &[u8],
    now_secs: i64,
) -> bool {
    // Fail-closed on any missing header. The Standard Webhooks spec requires all
    // three; accepting a request with a missing signature header would bypass
    // authentication entirely.
    let webhook_id = match header_str(headers, "webhook-id") {
        Some(v) => v,
        None => return false,
    };
    let webhook_ts = match header_str(headers, "webhook-timestamp") {
        Some(v) => v,
        None => return false,
    };
    let webhook_sig = match header_str(headers, "webhook-signature") {
        Some(v) => v,
        None => return false,
    };

    // Replay window. `webhook-timestamp` is ASCII unix seconds; if it is not a
    // valid integer the request is malformed → reject (fail-closed).
    let ts_secs: i64 = match webhook_ts.parse() {
        Ok(n) => n,
        Err(_) => return false,
    };
    if !timestamp_fresh(ts_secs, now_secs) {
        return false;
    }

    // Derive the key and compute the expected base64 digest over the signed
    // message id.timestamp.body.
    let key = standard_webhooks_key(secret);
    let mut mac = match HmacSha256::new_from_slice(&key) {
        Ok(m) => m,
        // HMAC accepts any non-empty key length; an empty key is rejected by the
        // crate. Treat it as a verification failure rather than panicking.
        Err(_) => return false,
    };
    mac.update(webhook_id.as_bytes());
    mac.update(b".");
    mac.update(webhook_ts.as_bytes());
    mac.update(b".");
    mac.update(body);
    let expected = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    // The signature header is a list of `v{version},{sig}` ENTRIES separated by
    // WHITESPACE (the real Standard Webhooks wire format that Polar uses), where
    // each entry uses a comma between the version and the signature itself —
    // i.e. `v1,<base64>` and, under key rotation, `v1,<sigA> v1,<sigB>`. (Splitting
    // on comma instead of whitespace corrupts the multi-entry rotation case, so
    // we must split on whitespace, then strip the `v1,`/`v1=` prefix per entry.)
    // Some emitters use `v1=` instead of `v1,`; we accept either prefix. Accept
    // the request if ANY entry's v1 signature matches the recomputed digest
    // (constant time). Only v1 is supported; entries lacking a recognised
    // version prefix are ignored (fail-closed on malformed headers).
    for entry in webhook_sig.split_whitespace() {
        let candidate = entry.strip_prefix("v1,").or_else(|| entry.strip_prefix("v1="));
        if let Some(sig) = candidate {
            if ct_eq(sig.as_bytes(), expected.as_bytes()) {
                return true;
            }
        }
    }
    false
}

/// Verify a Paddle Billing webhook signature (Ed25519).
///
/// Paddle signs `<timestamp><raw_body>` with its Ed25519 private key and sends
/// `Paddle-Signature: ts=<ts>;key1=<hexsig>`. We verify against the public key
/// configured via `PADDLE_PUBLIC_KEY` (32 raw bytes). Returns `true` only on a
/// valid signature over the exact bytes.
pub fn verify_ed25519(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let vk = match VerifyingKey::from_bytes(public_key) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let sig = Signature::from_bytes(signature);
    vk.verify(message, &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    // ── HMAC-SHA256 ───────────────────────────────────────────────────────

    #[test]
    fn hmac_known_answer_rfc4231_case1() {
        // RFC 4231 §4.2: key = 0x0b × 20, data = "Hi There".
        let key = [0x0bu8; 20];
        let tag = hmac_sha256_hex(&key, b"Hi There");
        assert_eq!(
            tag,
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn hmac_verify_accepts_correct_tag() {
        let key = b"whsec_test";
        let msg = b"{\"id\":\"evt_1\"}";
        let tag = hmac_sha256_hex(key, msg);
        assert!(verify_hmac_sha256_hex(key, msg, &tag));
    }

    #[test]
    fn hmac_verify_rejects_tampered_message() {
        let key = b"whsec_test";
        let tag = hmac_sha256_hex(key, b"original");
        // A single bit flipped in the body must invalidate the tag.
        assert!(!verify_hmac_sha256_hex(key, b"tampered", &tag));
    }

    #[test]
    fn hmac_verify_rejects_wrong_key() {
        let msg = b"body";
        let tag = hmac_sha256_hex(b"secret-a", msg);
        assert!(!verify_hmac_sha256_hex(b"secret-b", msg, &tag));
    }

    #[test]
    fn hmac_verify_rejects_bad_hex_and_length_mismatch() {
        let key = b"k";
        let msg = b"m";
        assert!(!verify_hmac_sha256_hex(key, msg, "nothex!!"));
        // A truncated tag is a length mismatch → false, not a panic.
        let full = hmac_sha256_hex(key, msg);
        assert!(!verify_hmac_sha256_hex(key, msg, &full[..10]));
    }

    // ── constant-time compare ─────────────────────────────────────────────

    #[test]
    fn ct_eq_equal_and_unequal() {
        assert!(ct_eq(b"abcdef", b"abcdef"));
        assert!(!ct_eq(b"abcdef", b"abcdeg"));
        assert!(!ct_eq(b"abc", b"abcdef")); // length mismatch → false
    }

    // ── freshness ─────────────────────────────────────────────────────────

    #[test]
    fn timestamp_fresh_within_window() {
        assert!(timestamp_fresh(1_000_000, 1_000_000));
        assert!(timestamp_fresh(1_000_000 + MAX_SKEW_SECS, 1_000_000));
        assert!(timestamp_fresh(1_000_000 - MAX_SKEW_SECS, 1_000_000));
    }

    #[test]
    fn timestamp_fresh_rejects_replay_outside_window() {
        assert!(!timestamp_fresh(1_000_000 + MAX_SKEW_SECS + 1, 1_000_000));
        assert!(!timestamp_fresh(1_000_000 - MAX_SKEW_SECS - 1, 1_000_000));
    }

    // ── header lookup ─────────────────────────────────────────────────────

    #[test]
    fn header_str_present_and_absent() {
        let mut h = HeaderMap::new();
        h.insert("Stripe-Signature", "t=1,v1=abc".parse().unwrap());
        assert_eq!(
            header_str(&h, "Stripe-Signature").as_deref(),
            Some("t=1,v1=abc")
        );
        assert!(header_str(&h, "Missing").is_none());
    }

    // ── Standard Webhooks (Polar) ─────────────────────────────────────────

    #[test]
    fn standard_webhooks_key_decodes_whsec_prefix() {
        let raw = [0x11u8; 32];
        let secret = format!(
            "whsec_{}",
            base64::engine::general_purpose::STANDARD.encode(raw)
        );
        assert_eq!(standard_webhooks_key(&secret), raw.to_vec());
    }

    #[test]
    fn standard_webhooks_key_accepts_bare_base64() {
        // No whsec_ prefix but still valid base64 → decode to the raw key.
        let raw = [0x22u8; 32];
        let secret = base64::engine::general_purpose::STANDARD.encode(raw);
        assert_eq!(standard_webhooks_key(&secret), raw.to_vec());
    }

    #[test]
    fn standard_webhooks_key_falls_back_to_raw_bytes() {
        // Not valid base64 and not a whsec_ secret → use the literal bytes.
        let s = "not-base64!!";
        assert_eq!(standard_webhooks_key(s), s.as_bytes().to_vec());
    }

    #[test]
    fn standard_webhooks_verifies_and_rejects() {
        use base64::engine::general_purpose::STANDARD;
        let raw_key = [7u8; 32];
        let secret = format!("whsec_{}", STANDARD.encode(raw_key));
        let now = 1_700_000_000i64;
        let id = "evt_1";
        let body = br#"{"type":"x"}"#;

        // Sign id.ts.body with HMAC-SHA256, base64-encode, set the 3 headers.
        let mut mac = HmacSha256::new_from_slice(&raw_key).unwrap();
        mac.update(format!("{id}.{now}.").as_bytes());
        mac.update(body);
        let sig = STANDARD.encode(mac.finalize().into_bytes());

        let mut h = HeaderMap::new();
        h.insert("webhook-id", id.parse().unwrap());
        h.insert("webhook-timestamp", now.to_string().parse().unwrap());
        h.insert("webhook-signature", format!("v1,{sig}").parse().unwrap());

        assert!(verify_standard_webhooks(&h, &secret, body, now));
        assert!(verify_standard_webhooks(&h, &secret, body, now + MAX_SKEW_SECS));

        // Outside the replay window → reject.
        assert!(!verify_standard_webhooks(&h, &secret, body, now + MAX_SKEW_SECS + 1));

        // Tampered body → reject.
        assert!(!verify_standard_webhooks(&h, &secret, b"{\"type\":\"y\"}", now));

        // Missing signature header → reject (fail-closed).
        let mut h2 = h.clone();
        h2.remove("webhook-signature");
        assert!(!verify_standard_webhooks(&h2, &secret, body, now));

        // Rotation: two v1 entries, second matches.
        let mut mac2 = HmacSha256::new_from_slice(&[9u8; 32]).unwrap();
        mac2.update(format!("{id}.{now}.").as_bytes());
        mac2.update(body);
        let sig2 = STANDARD.encode(mac2.finalize().into_bytes());
        let mut h3 = HeaderMap::new();
        h3.insert("webhook-id", id.parse().unwrap());
        h3.insert("webhook-timestamp", now.to_string().parse().unwrap());
        h3.insert(
            "webhook-signature",
            // Standard Webhooks SPACE-separates rotation entries; the comma lives
            // INSIDE each `v1,<sig>` entry. Mirror the real wire format here.
            format!("v1,{sig2} v1,{sig}").parse().unwrap(),
        );
        assert!(verify_standard_webhooks(&h3, &secret, body, now));
    }

    // ── Ed25519 (Paddle) ─────────────────────────────────────────────────

    #[test]
    fn ed25519_accepts_valid_signature() {
        use ed25519_dalek::{SigningKey, Signer};

        let seed = [7u8; 32]; // deterministic test key
        let sk = SigningKey::from_bytes(&seed);
        let vk = sk.verifying_key();
        let msg = b"1700000000{ \"data\": { \"id\": \"txn_1\" } }";
        let sig = sk.sign(msg);
        assert!(verify_ed25519(&vk.to_bytes(), msg, &sig.to_bytes()));
    }

    #[test]
    fn ed25519_rejects_tampered_message_and_wrong_key() {
        use ed25519_dalek::{SigningKey, Signer};

        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let vk = sk.verifying_key();
        let sig = sk.sign(b"1700000000body").to_bytes();

        // Tampered message → reject.
        assert!(!verify_ed25519(&vk.to_bytes(), b"1700000000BYTEM", &sig));
        // Wrong public key → reject.
        let other = SigningKey::from_bytes(&[99u8; 32])
            .verifying_key()
            .to_bytes();
        assert!(!verify_ed25519(&other, b"1700000000body", &sig));
        // Malformed public key bytes → reject (not panic).
        assert!(!verify_ed25519(&[0u8; 32], b"1700000000body", &sig));
    }
}
