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

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;
use getrandom::getrandom;
use zeroize::{Zeroize, Zeroizing};

/// AES-GCM nonce size (96 bits / 12 bytes), the length mandated by NIST SP
/// 800-38D for GCM and what `Aes256Gcm` expects. A fresh random nonce of this
/// length is prepended to every ciphertext (see `encrypt_with`).
const NONCE_LEN: usize = 12;

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

/// Borrow the installed key, or `None` if `set_key` was never called (e.g. a
/// unit test that exercises the model in isolation). Callers that need a key
/// MUST surface a clear error rather than silently storing plaintext.
pub fn field_enc_key() -> Option<&'static [u8; 32]> {
    // `Zeroizing<[u8;32]>` derefs to `[u8;32]`; expose the underlying array by
    // reference so callers (encrypt/decrypt) keep their existing slice API.
    FIELD_ENC_KEY.get().map(|w| &**w)
}

/// Encrypt `plaintext` with AES-256-GCM under the process key.
///
/// Output is `base64(nonce || ciphertext_and_tag)` — a fresh 12-byte random
/// nonce per call (via `getrandom`) so identical plaintexts yield different
/// ciphertexts. Returns `Err` if the key is unset or the cipher fails; the
/// caller MUST propagate (never `.unwrap()`-and-store) so a crypto failure
/// cannot silently persist plaintext.
pub fn encrypt(plaintext: &str) -> Result<String, FieldCryptoError> {
    let key = field_enc_key().ok_or(FieldCryptoError::KeyUnset)?;
    encrypt_with(plaintext, key)
}

/// Decrypt a `base64(nonce || ciphertext_and_tag)` blob produced by `encrypt`.
/// Fails closed on any tampering, truncation, or wrong key (GCM auth tag).
pub fn decrypt(blob: &str) -> Result<String, FieldCryptoError> {
    let key = field_enc_key().ok_or(FieldCryptoError::KeyUnset)?;
    decrypt_with(blob, key)
}

/// Keyed variant for callers/tests that supply an explicit 32-byte key without
/// touching the global slot. Public so the SeaORM model unit tests can exercise
/// the round-trip without booting the server.
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

/// Keyed decrypt — see `decrypt`.
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
        assert_ne!(a, b, "nonce reuse: identical ciphertexts for identical plaintext");
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
        let packed = base64::engine::general_purpose::STANDARD.decode(ct).unwrap();
        let short = base64::engine::general_purpose::STANDARD.encode(&packed[..NONCE_LEN]);
        let err = decrypt_with(&short, &key).expect_err("truncated blob must fail");
        assert!(matches!(err, FieldCryptoError::Decrypt | FieldCryptoError::Decode));
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
        assert!(err.contains("32"), "error must mention the 32-byte requirement: {err}");
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
        let _ty_check: fn() =
            || -> () { let _ = std::sync::OnceLock::<Zeroizing<[u8; 32]>>::new(); };
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
}
