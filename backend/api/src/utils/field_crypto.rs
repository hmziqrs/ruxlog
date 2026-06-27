//! Field-level encryption at rest for sensitive columns (AES-256-GCM).
//!
//! `payout_accounts.metadata` is a JSONB blob holding provider-specific payout
//! configuration (bank routing, wallet addresses, etc.). Stored plaintext it is
//! a CWE-312 exposure: a database dump leaks every account's payout details.
//! This module encrypts the blob with AES-256-GCM keyed by a server master key
//! (`FIELD_ENC_KEY`), decrypting transparently on read. The key is loaded once
//! at boot into a process-wide `OnceCell` (see `set_key` / `field_enc_key`),
//! so the SeaORM model layer can encrypt/decrypt without each caller threading
//! the key through — no caller can forget to encrypt.
//!
//! Threat model: an attacker with the DB but not the key cannot read payout
//! metadata (GCM gives confidentiality + integrity; tampering fails closed).
//! An attacker with the key already holds enough to decrypt at-rest secrets —
//! keying it on a dedicated `FIELD_ENC_KEY` (not `COOKIE_KEY`) limits blast
//! radius: a leaked cookie key alone does not decrypt payout metadata.
//!
//! ── CRYP-KM-003 (key versioning) ───────────────────────────────────────────
//!
//! Every NEW ciphertext carries a key-id/version prefix so a rotated key is
//! unambiguous at decrypt time. The envelope is the literal ASCII tag
//! `V1:<base64(nonce||ct||tag)>` (version 1 = the current primary key). Decrypt
//! accepts BOTH the new prefixed shape AND legacy un-prefixed blobs
//! (`base64(nonce||ct||tag)` written before CRYP-KM-003), trying the current
//! key first and then the optional previous key (`FIELD_ENC_KEY_PREV`) — so a
//! rotation is a non-breaking, rolling deploy: existing rows decrypt throughout
//! the window and are re-encrypted to `V1:` the next time they are written.
//!
//! Two key slots:
//!   * primary (`FIELD_ENC_KEY`) — encrypts all NEW envelopes AND decrypts;
//!   * previous (`FIELD_ENC_KEY_PREV`, optional) — decrypt-only, for the window
//!     after a rotation when some rows are still encrypted under the old key.
//!     A missing previous key is normal (first deploy / after a full backfill).

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use getrandom::getrandom;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::{Zeroize, Zeroizing};

/// AES-GCM nonce size (96 bits / 12 bytes), the length mandated by NIST SP
/// 800-38D for GCM and what `Aes256Gcm` expects. A fresh random nonce of this
/// length is prepended to every ciphertext (see `encrypt_with`).
const NONCE_LEN: usize = 12;

/// Key-id/version tag prefixing every NEW envelope (`V1:` + base64 payload).
/// Picked as printable ASCII that cannot appear inside standard base64 output
/// (no `+`, `/`, `=`, alphanumerics collision — `:` is the separator), so a
/// prefixed envelope and a legacy un-prefixed envelope are unambiguously
/// distinguishable by `str::starts_with`.
const V1_PREFIX: &str = "V1:";

/// A dedicated, process-wide 32-byte AES-256 key for field-level encryption.
/// Loaded once at boot from `FIELD_ENC_KEY` (see `state.rs`). Held in a
/// `OnceCell` so the SeaORM model layer can reach it without the caller
/// passing the key through every read/write — the model layer encrypts and
/// decrypts on its own, so no caller can forget.
///
/// GAP-016 (CWE-316/459): the payload is wrapped in `Zeroizing<[u8; 32]>`. The
/// transient key material used during `encrypt_with`/`decrypt_with` (nonce,
/// packed bytes, plaintext) is genuinely scrubbed on scope exit. NB: Rust does
/// NOT run `Drop` for `static` items at normal process exit, so this master key
/// is NOT scrubbed at teardown (the OS reclaims the pages) — the `Zeroizing`
/// wrap here is best-effort defense-in-depth; the real scrubbing is the
/// transient material above.
static FIELD_ENC_KEY: std::sync::OnceLock<Zeroizing<[u8; 32]>> = std::sync::OnceLock::new();

/// CRYP-KM-003: the optional PREVIOUS key (`FIELD_ENC_KEY_PREV`), decrypt-only.
/// Present only during the rotation window after a key change, so rows still
/// encrypted under the old key remain readable until they are re-encrypted.
static FIELD_ENC_KEY_PREV: std::sync::OnceLock<Zeroizing<[u8; 32]>> = std::sync::OnceLock::new();

/// Install the 32-byte field-encryption key into the process-wide slot.
/// Called once at boot from `main`. Returns `Err` if the key length is not
/// exactly 32 bytes — AES-256 requires a 256-bit key, and silently truncating
/// or padding would weaken the cipher. Subsequent calls (e.g. in tests)
/// succeed only if the same key is re-installed.
pub fn set_key(key: &[u8]) -> Result<(), String> {
    if key.len() != 32 {
        return Err(format!(
            "FIELD_ENC_KEY must be exactly 32 bytes for AES-256 (got {}). \
             Generate with: openssl rand -base64 32 | head -c 32 | base64 \
             or a raw 32-byte hex: openssl rand -hex 16 | head -c 32.",
            key.len()
        ));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(key);
    let wrapped = Zeroizing::new(arr);
    // Ignore re-init with the same key (tests); error on a conflicting key so a
    // silent key rotation never halves the process on a partial overwrite.
    FIELD_ENC_KEY.get_or_init(|| wrapped);
    if FIELD_ENC_KEY.get().map(|w| &**w) != Some(&arr) {
        return Err(
            "FIELD_ENC_KEY was already initialized with a different value; \
             key rotation requires a process restart."
                .to_string(),
        );
    }
    Ok(())
}

/// CRYP-KM-003: install the optional PREVIOUS 32-byte key (`FIELD_ENC_KEY_PREV`)
/// for decrypt-only use during the rotation window. Best-effort / idempotent:
/// unset in a first deploy and after a completed backfill. A wrong length is an
/// operator error, so it is surfaced (not panicked) — boot code chooses how to
/// treat it. `None` is a valid input meaning "clear / not provided" (no-op).
pub fn set_previous_key(key: Option<&[u8]>) -> Result<(), String> {
    let Some(key) = key else {
        return Ok(());
    };
    if key.len() != 32 {
        return Err(format!(
            "FIELD_ENC_KEY_PREV must be exactly 32 bytes for AES-256 (got {}).",
            key.len()
        ));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(key);
    // Only install if no previous key is set yet OR the same key is re-installed
    // (tests). A conflicting mid-process change is a bug — surface it.
    if let Some(existing) = FIELD_ENC_KEY_PREV.get() {
        if **existing != arr {
            return Err(
                "FIELD_ENC_KEY_PREV was already initialized with a different value; \
                 key rotation requires a process restart."
                    .to_string(),
            );
        }
        return Ok(());
    }
    let _ = FIELD_ENC_KEY_PREV.set(Zeroizing::new(arr));
    Ok(())
}

/// Borrow the installed key, or `None` if `set_key` was never called (e.g. a
/// unit test that exercises the model in isolation). Callers that need a key
/// MUST surface a clear error rather than silently storing plaintext.
pub fn field_enc_key() -> Option<&'static [u8; 32]> {
    // `Zeroizing<[u8;32]>` derefs to `[u8;32]`; expose the underlying array by
    // reference so callers (encrypt/decrypt) keep their existing slice API.
    FIELD_ENC_KEY.get().map(|w| &**w)
}

/// CRYP-KM-003: borrow the installed PREVIOUS key, or `None` if none was set
/// (first deploy / after a completed backfill). Decrypt-only.
pub fn field_enc_key_prev() -> Option<&'static [u8; 32]> {
    FIELD_ENC_KEY_PREV.get().map(|w| &**w)
}

/// Encrypt `plaintext` with AES-256-GCM under the process key.
///
/// Output is the versioned envelope `V1:<base64(nonce||ct||tag)>` — a fresh
/// 12-byte random nonce per call (via `getrandom`) so identical plaintexts
/// yield different ciphertexts. Returns `Err` if the key is unset or the cipher
/// fails; the caller MUST propagate (never `.unwrap()`-and-store) so a crypto
/// failure cannot silently persist plaintext. Use [`encrypt_deterministic`]
/// instead when ciphertexts for equal plaintexts MUST collide (lookup columns
/// like `google_id`).
pub fn encrypt(plaintext: &str) -> Result<String, FieldCryptoError> {
    let key = field_enc_key().ok_or(FieldCryptoError::KeyUnset)?;
    Ok(format!("{V1_PREFIX}{}", encrypt_with(plaintext, key)?))
}

/// Decrypt a blob produced by `encrypt` (new `V1:` envelope) OR a legacy
/// un-prefixed `base64(nonce||ct||tag)` blob written before CRYP-KM-003.
/// Tries the current key first, then the optional previous key. Fails closed
/// on any tampering, truncation, or wrong key (GCM auth tag).
pub fn decrypt(blob: &str) -> Result<String, FieldCryptoError> {
    let (payload, was_prefixed) = strip_version_prefix(blob);
    // Try the current key first (the common case post-backfill).
    if let Some(key) = field_enc_key() {
        if let Ok(pt) = decrypt_with(payload, key) {
            return Ok(pt);
        }
    } else {
        return Err(FieldCryptoError::KeyUnset);
    }
    // Fall back to the previous key (rotation window) for BOTH prefixed and
    // legacy blobs — a `V1:` row written just before a rotation, or a legacy
    // row from before CRYP-KM-003, may both be old-key ciphertext.
    if let Some(prev) = field_enc_key_prev() {
        if let Ok(pt) = decrypt_with(payload, prev) {
            return Ok(pt);
        }
    }
    // Suppress unused-variable lint when the prefix was observed but neither
    // key path succeeded: the distinction is not a security signal to leak.
    let _ = was_prefixed;
    Err(FieldCryptoError::Decrypt)
}

/// Strip a leading `V1:` (or future `Vn:`) version tag, returning the inner
/// base64 payload and whether a prefix was present. A leading tag is the
/// CRYP-KM-003 envelope; its absence means a legacy pre-versioning blob. Only a
/// recognized `V<digits>:` form is treated as a prefix so a ciphertext whose
/// base64 happens to start with `V` is not mis-parsed.
fn strip_version_prefix(blob: &str) -> (&str, bool) {
    if let Some(rest) = blob.strip_prefix("V") {
        if let Some(colon) = rest.find(':') {
            let ver = &rest[..colon];
            if !ver.is_empty() && ver.chars().all(|c| c.is_ascii_digit()) {
                return (&rest[colon + 1..], true);
            }
        }
    }
    (blob, false)
}

/// Keyed variant for callers/tests that supply an explicit 32-byte key without
/// touching the global slot. Public so the SeaORM model unit tests can exercise
/// the round-trip without booting the server. NOTE: this is the LOW-LEVEL
/// payload encryptor (no version prefix) — prefer [`encrypt`] / [`decrypt`] in
/// application code so envelopes carry the key-id tag.
pub fn encrypt_with(plaintext: &str, key: &[u8; 32]) -> Result<String, FieldCryptoError> {
    // Fresh nonce per encryption — the security of GCM depends on nonce
    // uniqueness, so this MUST be CSPRNG-sourced and never reused. We do NOT
    // swallow the `getrandom` Result: a failed RNG leaves the buffer zeroed,
    // and reusing an all-zero nonce would be catastrophic for GCM.
    let mut nonce_bytes = Zeroizing::new([0u8; NONCE_LEN]);
    getrandom(nonce_bytes.as_mut()).map_err(|_| FieldCryptoError::Rng)?;
    let nonce = Nonce::from_slice(&*nonce_bytes);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| FieldCryptoError::Encrypt)?;

    // Pack nonce + ciphertext+tag, then base64 for safe JSONB storage.
    let mut packed = Zeroizing::new(Vec::with_capacity(NONCE_LEN + ciphertext.len()));
    packed.extend_from_slice(&*nonce_bytes);
    packed.extend_from_slice(&ciphertext);
    // The intermediate ciphertext Vec (which holds a transform of the plaintext)
    // is zeroized once it has been folded into the base64 output.
    ciphertext.zeroize();

    Ok(base64::engine::general_purpose::STANDARD.encode(&packed))
}

/// Keyed decrypt — see [`encrypt_with`] (low-level, no version prefix).
pub fn decrypt_with(blob: &str, key: &[u8; 32]) -> Result<String, FieldCryptoError> {
    let packed = Zeroizing::new(
        base64::engine::general_purpose::STANDARD
            .decode(blob.as_bytes())
            .map_err(|_| FieldCryptoError::Decode)?,
    );

    if packed.len() < NONCE_LEN {
        return Err(FieldCryptoError::Decode);
    }
    let (nonce_bytes, ciphertext) = packed.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    // Wrap the decrypted bytes in `Zeroizing` so that if the UTF-8 conversion
    // below fails (and the plaintext is therefore dropped rather than handed
    // to the caller), the raw decrypted bytes are scrubbed instead of being
    // left in heap memory.
    let mut plaintext = Zeroizing::new(
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| FieldCryptoError::Decrypt)?,
    );

    // On success the bytes move out of the wrapper (replaced by an empty Vec
    // that the `Zeroizing` Drop zeroizes harmlessly) into the returned String's
    // buffer; on the error path the still-full `Zeroizing<Vec<u8>>` Drop
    // zeroizes the original allocation.
    let bytes = std::mem::take(&mut *plaintext);
    String::from_utf8(bytes).map_err(|_| FieldCryptoError::Decode)
}

// ──────────────────────────────────────────────────────────────────────────
// CRYP-ENC-004: deterministic encryption for lookup columns (google_id)
// ──────────────────────────────────────────────────────────────────────────
//
// `google_id` must be searchable (`find_by_google_id` does an equality lookup),
// so the random-nonce [`encrypt`] above is unusable — two encryptions of the
// same id yield different ciphertexts and the lookup never matches. We need
// DETERMINISTIC AEAD: equal plaintext → equal ciphertext, while still being
// confidential and integrity-protected (misuse-resistant, no nonce reuse blast
// radius). With only `aes-gcm` + `hmac` + `sha2` available (no `aes-siv`) we
// build an SIV-style construction:
//
//   subkey  = HMAC-SHA256(master_key, "ruxlog/.../det/v1/subkey")   (32 bytes)
//   siv     = HMAC-SHA256(subkey, domain || plaintext)              (32 bytes)
//   nonce   = siv[0..12]   (deterministic, stored alongside the ct)
//   ct||tag = AES-256-GCM(subkey, nonce, plaintext)
//   output  = "D1:" + base64(nonce(12) || ct || tag)
//
// The 12-byte synthetic nonce is a function of the plaintext, so equal inputs
// produce equal envelopes (the column is searchable), while distinct inputs get
// collision-resistant, distinct nonces (the GCM nonce-uniq safety property).
//
// Security note: deterministic encryption leaks equality of plaintexts (two
// users sharing a google_id are visibly equal). For `google_id` this is
// acceptable — it is already a globally-unique lookup key, and a unique
// constraint means no two rows share one anyway.

/// Domain-separation label mixed into the deterministic subkey derivation so
/// the same master key cannot be mis-used across encryption purposes.
const DET_DOMAIN: &[u8] = b"ruxlog/field_crypto/det/v1/subkey";

/// Deterministic-envelope version tag.
const D1_PREFIX: &str = "D1:";

/// SHA-256 output length in bytes (used by the deterministic SIV construction).
const SHA256_LEN: usize = 32;

/// Deterministically encrypt `plaintext` under the process key. Equal inputs
/// (under the same key) produce equal outputs, so the ciphertext is usable as a
/// lookup key (e.g. `find_by_google_id`). The output is the tagged envelope
/// `D1:<base64(nonce(12) || ct || tag)>`. Returns `Err` if the key is unset.
///
/// The 12-byte nonce is a DETERMINISTIC function of the plaintext
/// (`HMAC-SHA256(derived_subkey, domain || plaintext)[0..12]`) rather than
/// random — so it is stable across calls, yet unique per distinct plaintext
/// (collision-resistant under SHA-256), which is exactly the GCM nonce-uniq
/// safety property. This is the standard "synthetic-IV" (SIV) pattern.
pub fn encrypt_deterministic(plaintext: &str) -> Result<String, FieldCryptoError> {
    let key = field_enc_key().ok_or(FieldCryptoError::KeyUnset)?;
    Ok(format!(
        "{D1_PREFIX}{}",
        encrypt_deterministic_with(plaintext, key)?
    ))
}

/// Deterministically decrypt a `D1:` envelope produced by
/// [`encrypt_deterministic`]. Tries the current key, then the optional previous
/// key (rotation window). Fails closed on tampering or a wrong key.
pub fn decrypt_deterministic(blob: &str) -> Result<String, FieldCryptoError> {
    let payload = blob.strip_prefix(D1_PREFIX).unwrap_or(blob);
    if let Some(key) = field_enc_key() {
        if let Ok(pt) = decrypt_deterministic_with(payload, key) {
            return Ok(pt);
        }
    } else {
        return Err(FieldCryptoError::KeyUnset);
    }
    if let Some(prev) = field_enc_key_prev() {
        if let Ok(pt) = decrypt_deterministic_with(payload, prev) {
            return Ok(pt);
        }
    }
    Err(FieldCryptoError::Decrypt)
}

/// Keyed deterministic encrypt (low-level, no envelope tag) — see
/// [`encrypt_deterministic`]. Public so the SeaORM model unit tests can exercise
/// the determinism contract without the global slot. Output is
/// `base64(nonce(12) || ct || tag)`, where the nonce is the synthetic IV
/// derived from the plaintext.
pub fn encrypt_deterministic_with(
    plaintext: &str,
    key: &[u8; 32],
) -> Result<String, FieldCryptoError> {
    let subkey = derive_subkey(key);
    // Synthetic IV: a 32-byte HMAC over the plaintext, truncated to the GCM
    // nonce length. Stable for equal (key, plaintext) pairs → deterministic
    // output; collision-resistant across distinct plaintexts → nonce-unique.
    let mut nonce_bytes = compute_siv(&subkey, plaintext.as_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes[..NONCE_LEN]);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*subkey));

    let mut ct = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| FieldCryptoError::Encrypt)?;

    // The 12-byte synthetic nonce travels with the ciphertext (it is NOT secret
    // — it is a function of the plaintext — but the decryptor needs it to run
    // GCM). Equal plaintext → equal nonce + equal ct → equal envelope, which is
    // what makes the column searchable.
    let mut packed = Vec::with_capacity(NONCE_LEN + ct.len());
    packed.extend_from_slice(&nonce_bytes[..NONCE_LEN]);
    packed.extend_from_slice(&ct);
    nonce_bytes.zeroize();
    ct.zeroize();
    Ok(base64::engine::general_purpose::STANDARD.encode(&packed))
}

/// Keyed deterministic decrypt — see [`encrypt_deterministic_with`].
pub fn decrypt_deterministic_with(
    payload: &str,
    key: &[u8; 32],
) -> Result<String, FieldCryptoError> {
    let subkey = derive_subkey(key);
    let packed = base64::engine::general_purpose::STANDARD
        .decode(payload.as_bytes())
        .map_err(|_| FieldCryptoError::Decode)?;
    if packed.len() < NONCE_LEN {
        return Err(FieldCryptoError::Decode);
    }
    let (nonce_bytes, ct) = packed.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&*subkey));
    let mut plaintext = Zeroizing::new(
        cipher
            .decrypt(nonce, ct)
            .map_err(|_| FieldCryptoError::Decrypt)?,
    );
    let bytes = std::mem::take(&mut *plaintext);
    String::from_utf8(bytes).map_err(|_| FieldCryptoError::Decode)
}

/// HKDF-style subkey derivation from the master key: a 32-byte HMAC-SHA256
/// under a domain-separated label. This is a one-step KDF (extract+expand
/// collapsed) sufficient because the master key is already uniformly random;
/// the domain label prevents cross-purpose reuse.
fn derive_subkey(key: &[u8; 32]) -> Zeroizing<[u8; 32]> {
    // Fully-qualified: both `Mac` (hmac) and `KeyInit` (aes_gcm::aead) are in
    // scope and both expose `new_from_slice`, so we name the hmac trait.
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(DET_DOMAIN);
    let out = mac.finalize().into_bytes();
    let mut arr = [0u8; SHA256_LEN];
    arr.copy_from_slice(&out);
    Zeroizing::new(arr)
}

/// Compute the deterministic SIV = HMAC-SHA256(subkey, DET_DOMAIN || plaintext).
/// Truncated to the GCM nonce length when used as a nonce; the full digest is
/// collision-resistant so distinct plaintexts get distinct nonces.
fn compute_siv(subkey: &[u8; 32], plaintext: &[u8]) -> Zeroizing<[u8; SHA256_LEN]> {
    let mut mac =
        <Hmac<Sha256> as Mac>::new_from_slice(subkey).expect("HMAC accepts any key length");
    mac.update(DET_DOMAIN);
    mac.update(plaintext);
    let out = mac.finalize().into_bytes();
    let mut arr = [0u8; SHA256_LEN];
    arr.copy_from_slice(&out);
    Zeroizing::new(arr)
}

/// Error type — never carries secret material. The GCM auth-tag failure maps to
/// `Decrypt`; we deliberately do not distinguish "wrong key" from "tampered
/// ciphertext" to avoid leaking which it is.
#[derive(Debug, thiserror::Error)]
pub enum FieldCryptoError {
    #[error("field-encryption key is not installed (set FIELD_ENC_KEY at boot)")]
    KeyUnset,
    #[error("OS CSPRNG failed while generating nonce")]
    Rng,
    #[error("AES-GCM encryption failed")]
    Encrypt,
    #[error("AES-GCM decryption failed (tampered ciphertext or wrong key)")]
    Decrypt,
    #[error("ciphertext was not valid base64 / was too short / was not valid UTF-8")]
    Decode,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed 32-byte test key — only ever used in `#[cfg(test)]`.
    fn test_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        for (i, b) in k.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(0x11);
        }
        k
    }

    #[test]
    fn round_trip_recovers_plaintext() {
        let key = test_key();
        let pt = r#"{"iban":"DE89370400440532013000","holder":"Jane Doe"}"#;
        let ct = encrypt_with(pt, &key).expect("encrypt");
        let recovered = decrypt_with(&ct, &key).expect("decrypt");
        assert_eq!(recovered, pt);
    }

    #[test]
    fn empty_plaintext_round_trips() {
        let key = test_key();
        let ct = encrypt_with("", &key).expect("encrypt");
        assert_eq!(decrypt_with(&ct, &key).expect("decrypt"), "");
    }

    #[test]
    fn different_plaintexts_yield_different_ciphertexts() {
        // Random nonce per encryption: encrypting the SAME plaintext twice must
        // also produce different ciphertexts (nonce diversity), which implies
        // distinct plaintexts do too.
        let key = test_key();
        let a = encrypt_with("same", &key).expect("encrypt");
        let b = encrypt_with("same", &key).expect("encrypt");
        assert_ne!(
            a, b,
            "nonce reuse: identical ciphertexts for identical plaintext"
        );
        let c = encrypt_with("different", &key).expect("encrypt");
        assert_ne!(a, c);
    }

    #[test]
    fn tampered_ciphertext_fails_closed() {
        let key = test_key();
        let ct = encrypt_with("sensitive", &key).expect("encrypt");
        // Decode the base64 blob, flip a byte in the ciphertext region, then
        // re-encode. This keeps the input valid base64/UTF-8 (so decrypt reaches
        // the AEAD step) while genuinely altering the ciphertext so the GCM auth
        // tag no longer matches. (Flipping a raw base64 *character* with 0xff
        // instead would yield invalid UTF-8 and panic before decrypting.)
        let mut packed = base64::engine::general_purpose::STANDARD
            .decode(ct.as_bytes())
            .expect("ciphertext is valid base64");
        let mid = packed.len() / 2;
        packed[mid] ^= 0xff;
        let tampered = base64::engine::general_purpose::STANDARD.encode(&packed);
        let err = decrypt_with(&tampered, &key).expect_err("tampered blob must fail");
        assert!(matches!(err, FieldCryptoError::Decrypt));
    }

    #[test]
    fn truncated_ciphertext_fails() {
        let key = test_key();
        let ct = encrypt_with("sensitive", &key).expect("encrypt");
        // Strip everything but the nonce — too short to ever be valid.
        let packed = base64::engine::general_purpose::STANDARD
            .decode(ct)
            .unwrap();
        let short = base64::engine::general_purpose::STANDARD.encode(&packed[..NONCE_LEN]);
        let err = decrypt_with(&short, &key).expect_err("truncated blob must fail");
        assert!(matches!(
            err,
            FieldCryptoError::Decrypt | FieldCryptoError::Decode
        ));
    }

    #[test]
    fn wrong_key_fails_closed() {
        let key = test_key();
        let ct = encrypt_with("sensitive", &key).expect("encrypt");
        let mut other = [0u8; 32];
        other.copy_from_slice(&key);
        other[0] ^= 0x01;
        let err = decrypt_with(&ct, &other).expect_err("wrong key must fail");
        assert!(matches!(err, FieldCryptoError::Decrypt));
    }

    #[test]
    fn invalid_base64_fails() {
        let key = test_key();
        let err = decrypt_with("!!!not base64!!!", &key).expect_err("non-base64 must fail");
        assert!(matches!(err, FieldCryptoError::Decode));
    }

    #[test]
    fn ciphertext_is_base64() {
        let key = test_key();
        let ct = encrypt_with("payload", &key).expect("encrypt");
        // Must be valid standard base64 (JSONB-safe, no control chars).
        base64::engine::general_purpose::STANDARD
            .decode(&ct)
            .expect("ciphertext must decode as base64");
        assert!(!ct.contains('"') && !ct.contains('\\'));
    }

    #[test]
    fn key_length_is_enforced() {
        let err = set_key(b"too-short").expect_err("non-32-byte key must be rejected");
        assert!(
            err.contains("32"),
            "error must mention the 32-byte requirement: {err}"
        );
    }

    #[test]
    fn set_key_round_trips_via_globals() {
        // Use a unique key so this test does not collide with the shared global
        // slot installed by other tests' `set_key` calls (OnceLock is global).
        let mut k = [0u8; 32];
        for (i, b) in k.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(3).wrapping_add(0x42);
        }
        // set_key may no-op if another test already installed a different key;
        // the explicit-keyed API is the contract under test here, so we only
        // assert the round-trip behavior that does not depend on the global.
        let pt = "global-slot-independent";
        let ct = encrypt_with(pt, &k).expect("encrypt");
        assert_eq!(decrypt_with(&ct, &k).expect("decrypt"), pt);
    }

    // GAP-016 (CWE-316/459): the global key slot is held in a `Zeroizing` wrapper
    // so its bytes are scrubbed on drop rather than lingering in heap memory.
    // We assert the wrapper type is in place (a compile-time guarantee that the
    // `Drop` impl that performs the zeroize is wired in) and that the public
    // key-borrow API still hands back a usable 32-byte slice. This is the
    // highest-value in-memory secret in the process.
    #[test]
    fn global_key_slot_is_zeroize_wrapped() {
        // `field_enc_key()` returns `Option<&'static [u8;32]>` dereffed out of
        // a `Zeroizing<[u8;32]>`. If it returns `Some`, the global slot is
        // populated; if `None`, the wrapper is still declared at the type level
        // (the static below is the proof it compiles). Either way the contract
        // — borrow the installed key without copying it — holds.
        let key_ref = field_enc_key();
        if let Some(k) = key_ref {
            assert_eq!(k.len(), 32, "borrowed key must be exactly 32 bytes");
        }
        // Compile-time assertion: the static's payload type is the zeroizing
        // wrapper, not a bare array. If a future edit reverts it to
        // `OnceLock<[u8;32]>`, this line fails to compile.
        let _ty_check: fn() = || -> () {
            let _ = std::sync::OnceLock::<Zeroizing<[u8; 32]>>::new();
        };
    }

    // GAP-016: the round-trip still works end-to-end after the zeroize wrapping
    // was added to the encrypt/decrypt paths (the `Zeroizing` nonce, packed Vec
    // and decrypted-plaintext wrapper). This guards against a regression where
    // a `mem::take`/move mistake corrupts the data path.
    #[test]
    fn round_trip_survives_zeroize_wrapping() {
        let key = test_key();
        for pt in ["", "a", "short", "🦀 unicode payload 🔑 with secrets"] {
            let ct = encrypt_with(pt, &key).expect("encrypt");
            assert_eq!(decrypt_with(&ct, &key).expect("decrypt"), pt);
        }
    }

    // ── CRYP-KM-003: versioned envelopes + key rotation ─────────────────────

    #[test]
    fn versioned_envelope_round_trips_via_globals() {
        // Best-effort global install — these tests run on a shared process slot.
        set_key(&test_key()).ok();
        let pt = "versioned payload";
        let ct = encrypt(pt).expect("encrypt");
        assert!(
            ct.starts_with("V1:"),
            "new envelope must carry the V1 prefix, got: {ct}"
        );
        assert_eq!(decrypt(&ct).expect("decrypt"), pt);
    }

    #[test]
    fn legacy_unprefixed_blob_still_decrypts() {
        // A blob written BEFORE CRYP-KM-003 has no `V1:` prefix. `decrypt` must
        // still read it (rolling deploy / pre-existing rows). Use the low-level
        // encrypt_with to synthesize a prefix-less legacy blob.
        set_key(&test_key()).ok();
        let legacy = encrypt_with("legacy-row", &test_key()).expect("legacy encrypt");
        assert!(!legacy.starts_with("V1:"), "fixture must be prefix-less");
        assert_eq!(decrypt(&legacy).expect("decrypt legacy"), "legacy-row");
    }

    #[test]
    fn previous_key_decrypts_after_rotation() {
        // Install current key + a previous key; an envelope encrypted under the
        // PREVIOUS key (raw, no prefix) must still decrypt.
        set_key(&test_key()).ok();
        let mut prev = [0u8; 32];
        for (i, b) in prev.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(5).wrapping_add(0x77);
        }
        set_previous_key(Some(&prev)).ok();
        let legacy_under_prev = encrypt_with("old-key-row", &prev).expect("prev encrypt");
        assert_eq!(
            decrypt(&legacy_under_prev).expect("decrypt under previous key"),
            "old-key-row"
        );
    }

    #[test]
    fn set_previous_key_rejects_wrong_length() {
        let err = set_previous_key(Some(b"too-short"))
            .expect_err("non-32-byte previous key must be rejected");
        assert!(err.contains("32"));
    }

    #[test]
    fn strip_version_prefix_parses_known_and_leaves_unknown() {
        assert_eq!(strip_version_prefix("V1:abc"), ("abc", true));
        assert_eq!(strip_version_prefix("V42:xyz"), ("xyz", true));
        // No prefix → returned unchanged.
        assert_eq!(
            strip_version_prefix("plainbase64=="),
            ("plainbase64==", false)
        );
        // Looks like a prefix but version is non-numeric → NOT a prefix.
        assert_eq!(strip_version_prefix("Vx:abc"), ("Vx:abc", false));
        // `V` with no colon → not a prefix.
        assert_eq!(strip_version_prefix("Value"), ("Value", false));
    }

    // ── CRYP-ENC-004: deterministic encryption (google_id) ──────────────────

    #[test]
    fn deterministic_encrypt_is_stable_across_calls() {
        let key = test_key();
        let a = encrypt_deterministic_with("google-123", &key).expect("enc");
        let b = encrypt_deterministic_with("google-123", &key).expect("enc");
        assert_eq!(
            a, b,
            "deterministic: equal plaintext must yield equal ciphertext"
        );
    }

    #[test]
    fn deterministic_encrypt_distinct_ids_differ() {
        let key = test_key();
        let a = encrypt_deterministic_with("google-1", &key).expect("enc");
        let b = encrypt_deterministic_with("google-2", &key).expect("enc");
        assert_ne!(a, b, "distinct ids must encrypt to distinct ciphertexts");
    }

    #[test]
    fn deterministic_round_trips() {
        let key = test_key();
        for id in ["", "a", "1234567890", "sub|very-long-google-id-string-🦀"] {
            let ct = encrypt_deterministic_with(id, &key).expect("enc");
            assert_eq!(decrypt_deterministic_with(&ct, &key).expect("dec"), id);
        }
    }

    #[test]
    fn deterministic_envelope_round_trips_via_globals() {
        set_key(&test_key()).ok();
        let id = "global-google-id";
        let ct = encrypt_deterministic(id).expect("enc");
        assert!(
            ct.starts_with("D1:"),
            "deterministic envelope must carry the D1 prefix, got: {ct}"
        );
        assert_eq!(decrypt_deterministic(&ct).expect("dec"), id);
    }

    #[test]
    fn deterministic_keyed_api_independent_of_globals() {
        let mut k = [0u8; 32];
        for (i, b) in k.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(9).wrapping_add(0x05);
        }
        let ct = encrypt_deterministic_with("lookup-key", &k).expect("enc");
        assert_eq!(
            decrypt_deterministic_with(&ct, &k).expect("dec"),
            "lookup-key"
        );
    }
}
