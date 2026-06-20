use chrono::{DateTime, FixedOffset, Utc};
use getrandom::getrandom;
use hmac::{Hmac, Mac};
use sha1::Sha1;

/// Alphabet for Base32 encoding/decoding without padding (RFC 4648)
/// Default TOTP step in seconds
pub const DEFAULT_TOTP_STEP: u64 = 30;
/// Default TOTP digits
pub const DEFAULT_TOTP_DIGITS: u32 = 6;

/// Generates a new random Base32 (RFC 4648, no padding) secret.
///
/// Returns `None` only if the OS CSPRNG fails — the previous implementation
/// swallowed `getrandom`'s `Result`, which could leave the buffer zeroed and
/// yield a predictable (all-zero) TOTP secret. See plan Phase 2f.
/// Common sizes: 20 bytes (~160 bits)
pub fn generate_secret_base32(num_bytes: usize) -> Option<String> {
    let mut buf = vec![0u8; num_bytes];
    getrandom(&mut buf).ok()?;
    Some(data_encoding::BASE32_NOPAD.encode(&buf))
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
    let counter = now.timestamp().div_euclid(step as i64) as u64;

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

/// Verifies a TOTP code allowing a sliding window of steps (to account for
/// clock skew) and returns the RFC 6238 time-step counter that matched.
///
/// - window: number of steps to check before/after current (e.g., 1 checks [-1, 0, +1])
///
/// Returns `Some(counter)` when a candidate within the window matched, where
/// `counter` is the matched RFC 6238 counter = `floor(unix_seconds / step) + i`
/// for the matched step offset `i`. Returns `None` when nothing matched.
///
/// Callers that must block code replay (V-MED-6) read this counter, compare it
/// against the user's stored `two_fa_last_totp_counter`, and only accept +
/// persist counters strictly greater than the last-used one.
pub fn verify_totp_code_at(
    secret_base32: &str,
    code: &str,
    now: DateTime<FixedOffset>,
    step: u64,
    digits: u32,
    window: i64,
) -> Option<i64> {
    // Require numeric ASCII and correct length
    if code.len() != digits as usize || !code.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let secret = match data_encoding::BASE32_NOPAD.decode(secret_base32.as_bytes()) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let current_counter = now.timestamp().div_euclid(step as i64);

    for i in -window..=window {
        let counter = current_counter + i;

        let mut msg = [0u8; 8];
        for (idx, b) in (counter as u64).to_be_bytes().iter().enumerate() {
            msg[idx] = *b;
        }

        if let Some(candidate) = hmac_truncate_to_digits(&secret, &msg, digits) {
            if constant_time_eq(code.as_bytes(), candidate.as_bytes()) {
                return Some(counter);
            }
        }
    }

    None
}

/// Convenience: verify TOTP code for current time using defaults (step=30s, digits=6, window=1)
/// and return the matched counter (V-MED-6). Callers that persist the
/// last-used counter MUST use this variant; callers that only need the boolean
/// can use [`verify_totp_code_now_bool`].
pub fn verify_totp_code_now(secret_base32: &str, code: &str) -> Option<i64> {
    verify_totp_code_at(
        secret_base32,
        code,
        Utc::now().fixed_offset(),
        DEFAULT_TOTP_STEP,
        DEFAULT_TOTP_DIGITS,
        1,
    )
}

/// Boolean convenience wrapper over [`verify_totp_code_now`] for callers that
/// do not track the last-used counter (e.g. login-time only checks).
pub fn verify_totp_code_now_bool(secret_base32: &str, code: &str) -> bool {
    verify_totp_code_now(secret_base32, code).is_some()
}

/// Pure decision: is a freshly matched TOTP counter acceptable given the
/// last-used counter persisted for this user?
///
/// A counter is fresh when the user has never consumed one (`last == None` —
/// first-ever verify) OR when it is strictly greater than the last consumed
/// counter. `counter <= last` is a replay of an already-used code (V-MED-6).
pub fn is_fresh_counter(matched: i64, last: Option<i64>) -> bool {
    match last {
        None => true,
        Some(prev) => matched > prev,
    }
}

/// Returns the value to persist as the new last-used counter after accepting
/// `matched`. Always the matched counter itself — replay protection only ever
/// advances the watermark forward.
pub fn next_last(matched: i64, _last: Option<i64>) -> i64 {
    matched
}

/// Generate human-friendly backup codes.
///
/// Returns `None` only if the OS CSPRNG fails. See plan Phase 2f.
/// Default strength: 10 codes, each 12 characters as 4-4-4 (A-Z2-9 excluding ambiguous).
pub fn generate_backup_codes(count: usize) -> Option<Vec<String>> {
    (0..count).map(|_| generate_backup_code()).collect()
}

/// Hash a list of backup codes for storage.
///
/// Hashes are Argon2id PHC strings (via the same `password_auth` wrapper used
/// for login passwords), so plaintext is unrecoverable and not brute-forceable
/// at speed if the DB leaks — replacing the previous bare, unsalted SHA-256.
/// Callers MUST run this off the async worker thread (Argon2id is memory-hard).
/// See plan Phase 2f.
pub fn hash_backup_codes(codes: &[String]) -> Vec<String> {
    codes.iter().map(|c| hash_backup_code(c)).collect()
}

/// Attempt to consume a backup code:
/// - Returns `Some(updated_hashes)` with the consumed code removed on success.
/// - Returns `None` if the input code matches no stored hash.
///
/// Verifies each stored Argon2id PHC hash against the input. Callers MUST run
/// this off the async worker thread. See plan Phase 2f.
pub fn consume_backup_code(hashed_codes: &[String], input_code: &str) -> Option<Vec<String>> {
    for (pos, stored) in hashed_codes.iter().enumerate() {
        if password_auth::verify_password(input_code, stored).is_ok() {
            let mut updated = hashed_codes.to_vec();
            updated.remove(pos);
            return Some(updated);
        }
    }
    None
}

/// Generate a single human-friendly backup code in the form XXXX-XXXX-XXXX.
///
/// Selects each symbol via rejection sampling (no modulo bias) and propagates
/// CSPRNG failure. See plan Phase 2f.
fn generate_backup_code() -> Option<String> {
    // Exclude ambiguous characters: 0, 1, O, I, L
    const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

    let mut chars = [0u8; 12];
    for c in chars.iter_mut() {
        *c = ALPHABET[sample_index(ALPHABET.len())?];
    }

    Some(format!(
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
        chars[11] as char,
    ))
}

/// Hash a single backup code with Argon2id (PHC string).
fn hash_backup_code(code: &str) -> String {
    password_auth::generate_hash(code)
}

/// Uniformly sample an index in `[0, len)` via rejection sampling so the
/// distribution is unbiased for any `len`. Eliminates the modulo-bias finding
/// (the previous `% 31` over-weighted the first 8 symbols since `256 % 31 == 8`).
/// Returns `None` only on CSPRNG failure. See plan Phase 2f.
fn sample_index(len: usize) -> Option<usize> {
    // Largest multiple of `len` that fits in a u8; reject bytes at/above it so
    // the accepted range divides evenly by `len`.
    let limit = 256 - (256 % len);
    loop {
        let mut b = [0u8; 1];
        getrandom(&mut b).ok()?;
        if (b[0] as usize) < limit {
            return Some((b[0] as usize) % len);
        }
    }
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

/// Constant-time comparison to avoid timing attacks on backup-code verification.
///
/// Backed by `subtle::ConstantTimeEq` (audited) rather than a hand-rolled loop,
/// consistent with `services::billing::webhook_util::ct_eq`. A length mismatch
/// returns `false` immediately — backup codes are a fixed length, so the length
/// is not a secret and leaking it is not a timing oracle. See plan Phase 1c/2f.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_secret_generation_is_base32() {
        let s = generate_secret_base32(20).expect("CSPRNG available");
        assert!(!s.is_empty());
        assert!(data_encoding::BASE32_NOPAD.decode(s.as_bytes()).is_ok());
    }

    #[test]
    fn test_totp_roundtrip_now() {
        let secret = generate_secret_base32(20).expect("CSPRNG available");
        let code = generate_totp_code_now(&secret, DEFAULT_TOTP_DIGITS).unwrap();
        // verify_totp_code_now returns the matched counter on success.
        assert!(verify_totp_code_now(&secret, &code).is_some());
        assert!(verify_totp_code_now_bool(&secret, &code));
    }

    // ---- V-MED-6: counter math + replay decision ----

    #[test]
    fn test_verify_returns_current_counter_on_match() {
        // A code generated at a fixed instant must verify against that instant
        // and report the RFC 6238 counter = floor(unix_seconds / step).
        let secret = generate_secret_base32(20).expect("CSPRNG available");
        let now = Utc.timestamp_opt(1_700_000_045, 0).unwrap().fixed_offset(); // 45s
        let code =
            generate_totp_code_at(&secret, now, DEFAULT_TOTP_STEP, DEFAULT_TOTP_DIGITS).unwrap();
        let matched =
            verify_totp_code_at(&secret, &code, now, DEFAULT_TOTP_STEP, DEFAULT_TOTP_DIGITS, 1)
                .expect("code should verify at its own instant");
        // 1_700_000_045 / 30 = 56_666_668 (floor); the matched step is exactly that.
        assert_eq!(matched, 1_700_000_045_i64 / 30);
    }

    #[test]
    fn test_verify_returns_none_on_wrong_code() {
        let secret = generate_secret_base32(20).expect("CSPRNG available");
        let now = Utc.timestamp_opt(1_700_000_045, 0).unwrap().fixed_offset();
        assert!(verify_totp_code_at(
            &secret,
            "000000",
            now,
            DEFAULT_TOTP_STEP,
            DEFAULT_TOTP_DIGITS,
            1
        )
        .is_none());
    }

    // Pure replay-decision helpers — no DB needed.

    #[test]
    fn test_first_ever_verify_accepts_any_counter() {
        // last == None → any valid in-window counter is accepted (first use).
        assert!(is_fresh_counter(1_000_000, None));
        assert!(is_fresh_counter(0, None));
        assert!(is_fresh_counter(i64::MAX, None));
    }

    #[test]
    fn test_replay_of_used_counter_is_rejected() {
        // (a) A counter accepted once sets last_used.
        let last_after_first = next_last(1_000_000, None);
        assert_eq!(last_after_first, 1_000_000);

        // (b) The SAME counter submitted again within the window is rejected.
        assert!(!is_fresh_counter(1_000_000, Some(last_after_first)));
        // And so is an earlier one.
        assert!(!is_fresh_counter(999_999, Some(last_after_first)));
    }

    #[test]
    fn test_higher_counter_is_accepted_after_step_advances() {
        // (c) After the step advances, a fresh code at a higher counter is accepted.
        let last = Some(1_000_000_i64);
        assert!(is_fresh_counter(1_000_001, last));
        assert!(is_fresh_counter(1_000_500, last));
        assert!(is_fresh_counter(i64::MAX, last));
        // next_last advances the watermark forward to the matched value.
        assert_eq!(next_last(1_000_001, last), 1_000_001);
    }

    #[test]
    fn test_next_last_always_matches_accepted_counter() {
        // The watermark only ever moves to the freshly-accepted counter.
        assert_eq!(next_last(5, None), 5);
        assert_eq!(next_last(7, Some(5)), 7);
        assert_eq!(next_last(42, Some(100)), 42);
    }

    #[test]
    fn test_backup_codes_generation_and_hashing() {
        let codes = generate_backup_codes(5).expect("CSPRNG available");
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
            // Argon2id PHC string (replaces bare 64-char SHA-256 hex). See plan 2f.
            assert!(h.starts_with("$argon2"), "expected Argon2id PHC string, got: {h}");
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
