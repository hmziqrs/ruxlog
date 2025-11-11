use chrono::{DateTime, FixedOffset, Utc};
use getrandom::getrandom;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Digest, Sha256};

/// Alphabet for Base32 encoding/decoding without padding (RFC 4648)

/// Default TOTP step in seconds
pub const DEFAULT_TOTP_STEP: u64 = 30;
/// Default TOTP digits
pub const DEFAULT_TOTP_DIGITS: u32 = 6;

/// Generates a new random Base32 (RFC 4648, no padding) secret
/// Common sizes: 20 bytes (~160 bits)
pub fn generate_secret_base32(num_bytes: usize) -> String {
    let mut buf = vec![0u8; num_bytes];
    // Fill with OS randomness; leave zeros if it fails
    let _ = getrandom(&mut buf);

    data_encoding::BASE32_NOPAD.encode(&buf)
}

/// Builds an otpauth URI compatible with Google Authenticator
/// Example: otpauth://totp/Issuer:email@example.com?secret=BASE32&issuer=Issuer&algorithm=SHA1&digits=6&period=30
pub fn build_otpauth_url(label: &str, issuer: &str, secret_base32: &str, digits: u32) -> String {
    let safe_label = urlencoding::encode(&format!("{}:{}", issuer, label)).into_owned();
    let safe_issuer = urlencoding::encode(issuer).into_owned();
    format!(
        "otpauth://totp/{}?secret={}&issuer={}&algorithm=SHA1&digits={}&period={}",
        safe_label, secret_base32, safe_issuer, digits, DEFAULT_TOTP_STEP
    )
}

/// Generates a TOTP code for the given secret and timestamp.
/// - secret_base32: Base32 encoded secret (RFC 4648, no padding)
/// - now: timestamp to use
/// - step: timestep in seconds (typically 30)
/// - digits: number of digits (typically 6)
pub fn generate_totp_code_at(
    secret_base32: &str,
    now: DateTime<FixedOffset>,
    step: u64,
    digits: u32,
) -> Option<String> {
    let secret = data_encoding::BASE32_NOPAD
        .decode(secret_base32.as_bytes())
        .ok()?;
    let counter = (now.timestamp() as i64).div_euclid(step as i64) as u64;

    let mut msg = [0u8; 8];
    for (i, b) in counter.to_be_bytes().iter().enumerate() {
        msg[i] = *b;
    }

    let mut mac = Hmac::<Sha1>::new_from_slice(&secret).ok()?;
    mac.update(&msg);
    let hmac = mac.finalize().into_bytes();

    let offset = (hmac[19] & 0x0f) as usize;
    let bin_code = ((hmac[offset] as u32 & 0x7f) << 24)
        | ((hmac[offset + 1] as u32) << 16)
        | ((hmac[offset + 2] as u32) << 8)
        | (hmac[offset + 3] as u32);

    let modulo = pow10(digits);
    let code = bin_code % modulo;

    Some(format!("{:0width$}", code, width = digits as usize))
}

/// Convenience: generate TOTP code for current time (UTC fixed offset)
pub fn generate_totp_code_now(secret_base32: &str, digits: u32) -> Option<String> {
    generate_totp_code_at(
        secret_base32,
        Utc::now().fixed_offset(),
        DEFAULT_TOTP_STEP,
        digits,
    )
}

/// Verifies a TOTP code allowing a sliding window of steps (to account for clock skew).
/// - window: number of steps to check before/after current (e.g., 1 checks [-1, 0, +1])
pub fn verify_totp_code_at(
    secret_base32: &str,
    code: &str,
    now: DateTime<FixedOffset>,
    step: u64,
    digits: u32,
    window: i64,
) -> bool {
    // Require numeric ASCII and correct length
    if code.len() != digits as usize || !code.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let secret = match data_encoding::BASE32_NOPAD.decode(secret_base32.as_bytes()) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let current_counter = (now.timestamp() as i64).div_euclid(step as i64);

    for i in -window..=window {
        let counter = (current_counter + i) as u64;

        let mut msg = [0u8; 8];
        for (idx, b) in counter.to_be_bytes().iter().enumerate() {
            msg[idx] = *b;
        }

        if let Some(candidate) = hmac_truncate_to_digits(&secret, &msg, digits) {
            if constant_time_eq(code.as_bytes(), candidate.as_bytes()) {
                return true;
            }
        }
    }

    false
}

/// Convenience: verify TOTP code for current time using defaults (step=30s, digits=6, window=1)
pub fn verify_totp_code_now(secret_base32: &str, code: &str) -> bool {
    verify_totp_code_at(
        secret_base32,
        code,
        Utc::now().fixed_offset(),
        DEFAULT_TOTP_STEP,
        DEFAULT_TOTP_DIGITS,
        1,
    )
}

/// Generate human-friendly backup codes.
/// Default strength: 10 codes, each 12 characters as 4-4-4 (A-Z2-9 excluding ambiguous).
pub fn generate_backup_codes(count: usize) -> Vec<String> {
    (0..count).map(|_| generate_backup_code()).collect()
}

/// Hash a list of backup codes using SHA-256 (hex). Store these hashes server-side.
/// Always store only hashes; return values are hex-encoded lowercase strings.
pub fn hash_backup_codes(codes: &[String]) -> Vec<String> {
    codes.iter().map(hash_backup_code).collect()
}

/// Attempt to consume a backup code:
/// - Returns Some(updated_hashes) with the consumed code removed (by its hash) on success
/// - Returns None if the input code does not match any hash
pub fn consume_backup_code(hashed_codes: &[String], input_code: &str) -> Option<Vec<String>> {
    let input_hash = hash_backup_code(&input_code.to_string());
    if let Some(pos) = hashed_codes
        .iter()
        .position(|h| constant_time_eq(h.as_bytes(), input_hash.as_bytes()))
    {
        let mut updated = hashed_codes.to_vec();
        updated.remove(pos);
        Some(updated)
    } else {
        None
    }
}

/// Generate a single human-friendly backup code in the form XXXX-XXXX-XXXX
fn generate_backup_code() -> String {
    // Exclude ambiguous characters: 0, 1, O, I, L
    const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

    let mut chars = [0u8; 12];
    for c in &mut chars {
        let mut b = [0u8; 1];
        let _ = getrandom(&mut b);
        let idx = (b[0] as usize) % ALPHABET.len();
        *c = ALPHABET[idx];
    }

    format!(
        "{}{}{}{}-{}{}{}{}-{}{}{}{}",
        chars[0] as char,
        chars[1] as char,
        chars[2] as char,
        chars[3] as char,
        chars[4] as char,
        chars[5] as char,
        chars[6] as char,
        chars[7] as char,
        chars[8] as char,
        chars[9] as char,
        chars[10] as char,
        chars[11] as char
    )
}

/// Hash a single backup code using SHA-256 (hex, lowercase)
fn hash_backup_code(code: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compute HMAC-SHA1 and dynamically truncate to 'digits' decimal code
fn hmac_truncate_to_digits(secret: &[u8], msg: &[u8; 8], digits: u32) -> Option<String> {
    let mut mac = Hmac::<Sha1>::new_from_slice(secret).ok()?;
    mac.update(msg);
    let hmac = mac.finalize().into_bytes();

    let offset = (hmac[19] & 0x0f) as usize;
    let bin_code = ((hmac[offset] as u32 & 0x7f) << 24)
        | ((hmac[offset + 1] as u32) << 16)
        | ((hmac[offset + 2] as u32) << 8)
        | (hmac[offset + 3] as u32);

    let modulo = pow10(digits);
    let code = bin_code % modulo;

    Some(format!("{:0width$}", code, width = digits as usize))
}

/// Returns 10^n for small n (n <= 10 is typical)
fn pow10(n: u32) -> u32 {
    // Avoid powf; compute safely for reasonable digit bounds
    let mut v = 1u32;
    for _ in 0..n {
        v = v.saturating_mul(10);
    }
    v
}

/// Constant-time comparison to avoid timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_generation_is_base32() {
        let s = generate_secret_base32(20);
        assert!(!s.is_empty());
        assert!(data_encoding::BASE32_NOPAD.decode(s.as_bytes()).is_ok());
    }

    #[test]
    fn test_totp_roundtrip_now() {
        let secret = generate_secret_base32(20);
        let code = generate_totp_code_now(&secret, DEFAULT_TOTP_DIGITS).unwrap();
        assert!(verify_totp_code_now(&secret, &code));
    }

    #[test]
    fn test_backup_codes_generation_and_hashing() {
        let codes = generate_backup_codes(5);
        assert_eq!(codes.len(), 5);
        for c in &codes {
            assert_eq!(c.len(), 14); // 12 chars + 2 hyphens
            assert!(c
                .chars()
                .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '-'));
        }
        let hashes = hash_backup_codes(&codes);
        assert_eq!(hashes.len(), 5);
        for h in &hashes {
            assert_eq!(h.len(), 64);
        }

        let updated = consume_backup_code(&hashes, &codes[0]);
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().len(), 4);

        let not_found = consume_backup_code(&hashes, "WRONG-CODE-0000");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_otpauth_url_format() {
        let url = build_otpauth_url("user@example.com", "Ruxlog", "SECRET", 6);
        assert!(url.starts_with("otpauth://totp/"));
        assert!(url.contains("issuer=Ruxlog"));
        assert!(url.contains("digits=6"));
        assert!(url.contains("period=30"));
    }
}
