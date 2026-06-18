//! Shared primitives for webhook signature verification.
//!
//! Every billing provider reads its signature header(s) from the incoming
//! request and authenticates the raw body. Previously each provider carried its
//! own hand-rolled `constant_time_eq_str` (9 copies) and recomputed HMAC-SHA256
//! inline; this module centralises the constant-time comparison, the HMAC
//! computation, and the Ed25519 verification Paddle needs. See plan Phase 1c.

use axum::http::HeaderMap;
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
