//! HMAC-SHA256 hashing for short-lived verification/reset codes.
//!
//! Forgot-password and email-verification codes used to be stored in plaintext,
//! so a database dump exposed every active code (crypto audit: "brute-forceable
//! plaintext reset codes"). We now store only `HMAC-SHA256(secret_key, code)`,
//! keyed by the server's `COOKIE_KEY`-derived secret (`AppState::secret_key`),
//! and look codes up by their deterministic hash.
//!
//! Threat model: an attacker who has the DB but not the secret cannot recover
//! or brute-force a stored code (the rate limiter caps online guessing too —
//! see Phase 3c). An attacker who has the secret already holds the
//! cookie-signing key and can forge sessions outright, so keying the code hash
//! on the same secret is not an additional exposure.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Compute `HMAC-SHA256(secret_key, code)` as lowercase hex.
///
/// Deterministic by design: the same plaintext always yields the same hash, so
/// a stored code can be found via an indexed `WHERE code_hash = ?` lookup rather
/// than a per-row Argon2 verify.
pub fn hash_code(secret_key: &[u8], code: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key).expect("HMAC accepts any key length");
    mac.update(code.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let key = b"server-secret";
        assert_eq!(hash_code(key, "AB12cd"), hash_code(key, "AB12cd"));
    }

    #[test]
    fn hash_differs_by_code_and_key() {
        let k1 = b"server-secret";
        let k2 = b"other-secret";
        assert_ne!(hash_code(k1, "AAAAAA"), hash_code(k1, "BBBBBB"));
        assert_ne!(hash_code(k1, "AAAAAA"), hash_code(k2, "AAAAAA"));
    }

    #[test]
    fn hash_is_hex() {
        let h = hash_code(b"k", "code");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h.len(), 64); // SHA-256 = 32 bytes = 64 hex chars
    }
}
