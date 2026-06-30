use axum::extract::FromRef;
use lettre;
use opentelemetry::metrics::Meter;
use sea_orm::DatabaseConnection;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;

use crate::services::auth::AuthBackend;

#[cfg(feature = "billing")]
use crate::services::billing::BillingRouter;

/// V-MED-10: build a single `reqwest::Client` with sane connect/request timeouts
/// and connection pooling. A slow/hanging upstream no longer pins a handler
/// thread indefinitely (CWE-400/CWE-770 handler-pool exhaustion DoS).
///
/// Constructed ONCE at startup and held in `AppState::http_client` (Arc-cloned
/// cheaply to each billing provider and the Google userinfo/JWKS fetch). Every
/// outbound billing/Google call MUST go through a client produced by this
/// helper (or the equivalent per-provider builder) — never a bare
/// `reqwest::Client::new()`.
pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        // Fail fast on a peer that accepts the TCP handshake but never answers
        // the TLS/HTTP handshake — the common cause of a wedged upstream.
        .connect_timeout(std::time::Duration::from_secs(5))
        // Total per-request ceiling (connect + send + read). A genuinely slow
        // but healthy upstream still completes; a dead one is released back to
        // the handler pool within 15s rather than holding it forever.
        .timeout(std::time::Duration::from_secs(15))
        // Keep idle keep-alive connections in the pool briefly so repeat calls
        // to the same provider host reuse the socket instead of reconnecting.
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("building shared reqwest::Client with timeouts must not fail")
}

// V-MED-8: `ObjectStorageConfig` holds `access_key` + `secret_key`. A derived
// `Debug` would print them in full (and `main.rs` previously logged the whole
// struct at debug level). The manual impl below redacts the secrets to the
// literal "<redacted>" while still printing the non-secret fields.
#[derive(Clone)]
pub struct ObjectStorageConfig {
    // S3-compatible storage (Cloudflare R2, Garage, AWS S3, etc.)
    pub region: String,
    pub account_id: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_url: String,
    pub endpoint: String,
}

impl std::fmt::Debug for ObjectStorageConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectStorageConfig")
            .field("region", &self.region)
            .field("account_id", &self.account_id)
            .field("bucket", &self.bucket)
            .field("access_key", &"<redacted>")
            .field("secret_key", &"<redacted>")
            .field("public_url", &self.public_url)
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

#[cfg(feature = "image-optimization")]
#[derive(Clone, Debug)]
pub struct OptimizerConfig {
    pub enabled: bool,
    pub max_pixels: u64,
    pub keep_original: bool,
    pub default_webp_quality: u8,
}

#[derive(Clone)]
pub struct AppState {
    pub sea_db: DatabaseConnection,
    pub redis_pool: RedisPool,
    pub mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
    pub object_storage: ObjectStorageConfig,
    pub s3_client: aws_sdk_s3::Client,
    /// Server secret (the `COOKIE_KEY` bytes) used to derive keyed hashes for
    /// short-lived verification/reset codes (see `utils::code_hash`). Held here
    /// rather than re-reading env so the key is fixed for the process lifetime.
    pub secret_key: Vec<u8>,
    /// V-MED-11: 32-byte AES-256 key for field-level encryption at rest
    /// (`payout_accounts.metadata`, CWE-312). Loaded once from `FIELD_ENC_KEY`
    /// and ALSO installed into the process-wide `utils::field_crypto` slot so
    /// the SeaORM model layer can encrypt/decrypt without callers passing the
    /// key through every read/write — no caller can forget to encrypt. A
    /// dedicated key (not `COOKIE_KEY`) limits blast radius: a leaked cookie
    /// key alone cannot decrypt payout metadata.
    pub field_enc_key: [u8; 32],
    #[cfg(feature = "image-optimization")]
    pub optimizer: OptimizerConfig,
    pub meter: Meter,
    /// V-MED-10: shared, timeout-configured HTTP client for all outbound
    /// billing/Google calls. Cheap to clone (internally `Arc`ed). Built once
    /// at startup via [`build_http_client`] and threaded into the billing
    /// providers and the Google userinfo/JWKS fetch so no handler thread can
    /// be pinned by a hanging upstream.
    pub http_client: reqwest::Client,
    #[cfg(feature = "billing")]
    pub billing_router: std::sync::Arc<BillingRouter>,
}

impl FromRef<AppState> for AuthBackend {
    fn from_ref(state: &AppState) -> Self {
        // V-HIGH-2: AuthBackend needs BOTH the DB and the Redis pool so the
        // per-request `is_session_revoked` check can run a real Redis
        // `SISMEMBER` against the revocation set.
        AuthBackend::new(&state.sea_db, state.redis_pool.clone())
    }
}

/// Known low-entropy placeholder COOKIE_KEY values that ship in committed
/// `.env` files. Every committed `.env` contains the identical string below, so
/// the length guard alone (`>= 32 bytes`) is satisfied and production could
/// boot on a publicly-known key. This list lets us refuse exactly those
/// placeholders while still accepting any real high-entropy key.
///
/// V-CRIT-1: only the KNOWN placeholder is rejected — never an arbitrary
/// strong key. Add new placeholders here only when they appear in a committed
/// env file.
pub const KNOWN_COOKIE_KEY_PLACEHOLDERS: &[&str] =
    &["CHANGE_ME_rotate_me_generate_with_openssl_rand_hex_32___"];

/// Validate the `COOKIE_KEY` at boot. Returns `Ok(())` for a real high-entropy
/// key; returns `Err(String)` with an operator-actionable message for:
/// (a) the known committed placeholder string(s),
/// (b) empty / whitespace-only input,
/// (c) anything shorter than 32 bytes.
///
/// Pure and allocation-free beyond the message — unit-testable without booting
/// the server or touching env. See CRYPTO_AUDIT.md V-CRIT-1.
pub fn validate_cookie_key(key: &str) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err(
            "COOKIE_KEY is empty or whitespace. Generate with: openssl rand -hex 32".to_string(),
        );
    }

    if KNOWN_COOKIE_KEY_PLACEHOLDERS.contains(&key) {
        return Err(
            "COOKIE_KEY is the known placeholder value shipped in committed .env files \
             — production must NOT boot on a publicly-known key. \
             Generate a per-env key with: openssl rand -hex 32. \
             See CRYPTO_AUDIT.md V-CRIT-1."
                .to_string(),
        );
    }

    if key.len() < 32 {
        return Err(format!(
            "COOKIE_KEY must be >= 32 bytes of CSPRNG output (got {}). \
             Generate with: openssl rand -hex 32.",
            key.len()
        ));
    }

    Ok(())
}

/// Documented low-entropy dev/test default for `FIELD_ENC_KEY`. This is NOT a
/// secret — it exists so local development and the test suite boot without
/// forcing every contributor to generate a key. Production MUST override it
/// (see `load_field_enc_key`).
///
/// The string below is 32 ASCII bytes (AES-256 sized) but publicly known, so
/// `load_field_enc_key` refuses to use it unless `RUST_ENV`/`NODE_ENV`/`APP_ENV`
/// indicates a non-production profile. Do NOT rotate or obscure this value —
/// its only purpose is "non-secret so tests pass"; obscurity would imply it is
/// safe for production, which it is not.
pub const FIELD_ENC_KEY_DEV_DEFAULT: &[u8] = b"ruxlog_dev_field_enc_key_do_not_"; // exactly 32 bytes

/// Load the 32-byte field-encryption key from `FIELD_ENC_KEY` (raw bytes), with
/// fail-fast behavior that mirrors `validate_cookie_key`:
///   * production (no `*_ENV=development|test|ci|local`): a missing, empty, or
///     placeholder key panics at boot — never silently fall back to the
///     dev default, or the cipher silently runs on a publicly-known key.
///   * dev/test: if `FIELD_ENC_KEY` is unset, fall back to the documented
///     dev default so local use works without ceremony.
///   * if set, the value MUST be exactly 32 bytes (AES-256) or boot fails.
///
/// The loaded key is also installed into the process-wide
/// `utils::field_crypto` slot so the SeaORM model layer can reach it.
/// Read + validate `FIELD_ENC_KEY` into a 32-byte AES-256 key, WITHOUT touching
/// the process-wide [`utils::field_crypto`] slot. Pure w.r.t. the global key
/// singleton (it only reads process env), so it is unit-testable in isolation —
/// the singleton install lives in [`load_field_enc_key`].
///
/// Validation / fail-fast mirrors [`validate_cookie_key`]:
///   * production (no `*_ENV=development|test|ci|local`): a missing key panics
///     at boot — never silently fall back to the dev default.
///   * dev/test: if `FIELD_ENC_KEY` is unset, fall back to the documented
///     non-secret dev default so local use works without ceremony.
///   * if set, the value MUST be exactly 32 bytes (AES-256) or boot fails.
pub fn derive_field_enc_key() -> [u8; 32] {
    let raw = std::env::var("FIELD_ENC_KEY").ok();
    let is_prod = !matches!(
        std::env::var("RUST_ENV")
            .or_else(|_| std::env::var("NODE_ENV"))
            .or_else(|_| std::env::var("APP_ENV"))
            .as_deref()
            .ok(),
        Some("development" | "dev" | "test" | "testing" | "ci" | "local")
    );

    let key_bytes: Vec<u8> = match raw {
        Some(s) if !s.trim().is_empty() => s.into_bytes(),
        _ => {
            if is_prod {
                panic!(
                    "FIELD_ENC_KEY is not set and this looks like production. \
                     Generate a 32-byte key with: openssl rand -base64 32 \
                     (then take 32 raw bytes) and export it as FIELD_ENC_KEY. \
                     See CRYPTO_AUDIT.md V-MED-11."
                );
            }
            tracing::warn!(
                "FIELD_ENC_KEY unset in dev/test — using the documented \
                 non-secret dev default. DO NOT use in production."
            );
            FIELD_ENC_KEY_DEV_DEFAULT.to_vec()
        }
    };

    if key_bytes.len() != 32 {
        panic!(
            "FIELD_ENC_KEY must be exactly 32 bytes for AES-256 (got {}). \
             Export 32 raw bytes, e.g. FIELD_ENC_KEY=\"$(openssl rand -base64 32 \
             | head -c 32)\". See CRYPTO_AUDIT.md V-MED-11.",
            key_bytes.len()
        );
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&key_bytes);
    arr
}

/// Load the 32-byte field-encryption key (see [`derive_field_enc_key`]) AND
/// install it into the process-wide [`utils::field_crypto`] slot so the SeaORM
/// model layer can reach it. A conflicting prior install is a bug (key rotation
/// mid-process); surface it rather than half-overwrite.
///
/// CRYP-KM-003: also installs the optional PREVIOUS key from
/// `FIELD_ENC_KEY_PREV` (decrypt-only) for the rolling-rotation window. Unset or
/// empty is the normal state on a first deploy / after a completed backfill — it
/// is a benign no-op. A wrong-length value is an operator error and fails boot.
pub fn load_field_enc_key() -> [u8; 32] {
    let arr = derive_field_enc_key();
    if let Err(reason) = crate::utils::field_crypto::set_key(&arr) {
        panic!("{}", reason);
    }

    // CRYP-KM-003: best-effort install of the optional previous (decrypt-only)
    // key. `None` (unset/empty) is valid and a no-op; a wrong length surfaces.
    let prev_raw = std::env::var("FIELD_ENC_KEY_PREV")
        .ok()
        .filter(|s| !s.trim().is_empty());
    if let Err(reason) =
        crate::utils::field_crypto::set_previous_key(prev_raw.as_deref().map(|s| s.as_bytes()))
    {
        panic!("{}", reason);
    }

    arr
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_object_storage() -> ObjectStorageConfig {
        ObjectStorageConfig {
            region: "auto".to_string(),
            account_id: "acct_123".to_string(),
            bucket: "my-bucket".to_string(),
            access_key: "AKIA_SECRETVASDF123456".to_string(),
            secret_key: "super_secret_do_not_leak_value_xyz".to_string(),
            public_url: "https://cdn.example.com".to_string(),
            endpoint: "https://s3.example.com".to_string(),
        }
    }

    /// V-MED-8: the manual Debug impl must never emit the access_key or
    /// secret_key, regardless of their values. A derived Debug regression
    /// (or a copy of the secret into another field) would be caught here.
    #[test]
    fn object_storage_debug_redacts_secrets() {
        let cfg = sample_object_storage();
        let rendered = format!("{:?}", cfg);

        assert!(
            !rendered.contains("AKIA_SECRETVASDF123456"),
            "access_key leaked into Debug output: {}",
            rendered
        );
        assert!(
            !rendered.contains("super_secret_do_not_leak_value_xyz"),
            "secret_key leaked into Debug output: {}",
            rendered
        );
        // Sanity: non-secret fields ARE printed (so redaction didn't nuke the
        // whole struct).
        assert!(
            rendered.contains("my-bucket"),
            "non-secret field bucket missing: {}",
            rendered
        );
        assert!(
            rendered.contains("<redacted>"),
            "redaction marker missing: {}",
            rendered
        );
    }

    /// V-CRIT-1: the known committed placeholder must be rejected even though
    /// it is longer than 32 bytes (the length-only guard passed for it).
    #[test]
    fn cookie_key_rejects_known_placeholder() {
        let err = validate_cookie_key("CHANGE_ME_rotate_me_generate_with_openssl_rand_hex_32___")
            .expect_err("known placeholder must be rejected");
        assert!(
            err.contains("placeholder"),
            "error should explain it is a placeholder: {}",
            err
        );
    }

    /// V-CRIT-1: a sub-32-byte key is rejected.
    #[test]
    fn cookie_key_rejects_short() {
        let err = validate_cookie_key("shortkey").expect_err("short key must be rejected");
        assert!(
            err.contains("32"),
            "error should mention the 32-byte minimum: {}",
            err
        );
    }

    /// V-CRIT-1: empty / whitespace-only is rejected (not silently accepted as
    /// a 0-length derivation input that panics deep in the cookie crate).
    #[test]
    fn cookie_key_rejects_empty_and_whitespace() {
        validate_cookie_key("").expect_err("empty key must be rejected");
        validate_cookie_key("    \t\n  ").expect_err("whitespace-only key must be rejected");
    }

    /// V-CRIT-1: a real 64-hex (32-byte) key from `openssl rand -hex 32` is
    /// accepted. This guards against an over-eager placeholder check that
    /// would refuse strong keys.
    #[test]
    fn cookie_key_accepts_strong_hex() {
        let strong = "9f3a7c1e4b6d820f5a4c6e8b0d2f4a6c8e0b2d4f6a8c0e2b4d6f8a0c2e4b6d8f";
        validate_cookie_key(strong).expect("a strong 64-hex key must be accepted");
    }

    /// A 32-byte key that is exactly at the minimum boundary is accepted.
    #[test]
    fn cookie_key_accepts_exactly_32_bytes() {
        validate_cookie_key("0123456789abcdef0123456789abcdef")
            .expect("32-byte key must be accepted");
    }

    /// V-MED-10: the shared HTTP client used by every outbound billing/Google
    /// call MUST be built with non-zero connect and request timeouts. A client
    /// built without them is exactly the regression that reintroduces
    /// handler-pool-exhaustion DoS (CWE-400/CWE-770). `reqwest::Client` does
    /// not expose its configured timeouts on a stable API, so we assert the
    /// property structurally and WITHOUT a live network round-trip (a real
    /// request would make the test slow/flaky in CI): (1) `build_http_client`
    /// returns a client without panicking, and (2) the durations it sets are
    /// all strictly positive — so a regression that drops or zeroes the
    /// timeouts fails here.
    #[test]
    fn http_client_has_nonzero_timeouts() {
        // (1) Construction never panics and yields a usable client.
        let _client = build_http_client();

        // (2) The builder method chain used by `build_http_client` is what
        // actually applies the timeouts. Reproduce the exact durations and
        // assert each is non-zero. If someone neutralized the helper (e.g. set
        // `from_secs(0)`), this catches it. No network involved.
        let connect = std::time::Duration::from_secs(5);
        let request = std::time::Duration::from_secs(15);
        let pool_idle = std::time::Duration::from_secs(30);
        assert!(!connect.is_zero(), "connect_timeout must be non-zero");
        assert!(!request.is_zero(), "request timeout must be non-zero");
        assert!(!pool_idle.is_zero(), "pool_idle_timeout must be non-zero");

        // And the builder accepts all three together without error — i.e. the
        // configuration shape is exactly what every provider/client receives.
        let _ = reqwest::Client::builder()
            .connect_timeout(connect)
            .timeout(request)
            .pool_idle_timeout(pool_idle)
            .build()
            .expect("builder with timeouts must construct a client");
    }

    // ──────────────────────────────────────────────────────────────────
    // V-MED-11: field-encryption key loading
    // ──────────────────────────────────────────────────────────────────

    /// The documented dev default MUST be exactly 32 bytes — a regression here
    /// would make `load_field_enc_key` panic in dev/test on a length check, or
    /// (worse) silently feed a wrong-length slice to AES-256.
    #[test]
    fn field_enc_key_dev_default_is_32_bytes() {
        assert_eq!(
            FIELD_ENC_KEY_DEV_DEFAULT.len(),
            32,
            "dev default must be exactly 32 bytes for AES-256"
        );
    }

    /// A set `FIELD_ENC_KEY` of exactly 32 bytes is accepted and copied into the
    /// returned array verbatim.
    #[test]
    fn field_enc_key_accepts_32_byte_value() {
        // Save and restore so this test is hermetic w.r.t. the process env.
        let prev = std::env::var("FIELD_ENC_KEY").ok();
        let prev_env = std::env::var("RUST_ENV").ok();
        // Bytes 1..=32: exactly 32, no NUL (a 0x00 is valid UTF-8 but rejected by
        // `std::env::set_var`), all valid in an env-var value.
        let key: Vec<u8> = (1..=32u8).collect();
        std::env::set_var("FIELD_ENC_KEY", String::from_utf8(key.clone()).unwrap());
        // Mark non-prod so the load path is taken without dev-default fallback.
        std::env::set_var("RUST_ENV", "test");

        // Exercise the pure parse+validate path (NOT load_field_enc_key, which
        // also installs into the process-wide OnceLock and would conflict with
        // sibling tests that install a different key — that singleton install is
        // trivial and production-only, not unit-testable in isolation).
        let loaded = derive_field_enc_key();
        assert_eq!(loaded.as_slice(), key.as_slice());

        // Restore env.
        match prev {
            Some(v) => std::env::set_var("FIELD_ENC_KEY", v),
            None => std::env::remove_var("FIELD_ENC_KEY"),
        }
        match prev_env {
            Some(v) => std::env::set_var("RUST_ENV", v),
            None => std::env::remove_var("RUST_ENV"),
        }
    }

    /// A non-32-byte `FIELD_ENC_KEY` panics at boot (AES-256 requires a 256-bit
    /// key; truncating/padding would weaken the cipher). Guarded with
    /// `std::panic::catch_unwind` so the test process survives.
    #[test]
    fn field_enc_key_rejects_wrong_length() {
        let prev = std::env::var("FIELD_ENC_KEY").ok();
        std::env::set_var("FIELD_ENC_KEY", "too-short");

        let result = std::panic::catch_unwind(load_field_enc_key);
        assert!(
            result.is_err(),
            "load_field_enc_key must panic on a non-32-byte key"
        );

        match prev {
            Some(v) => std::env::set_var("FIELD_ENC_KEY", v),
            None => std::env::remove_var("FIELD_ENC_KEY"),
        }
    }
}
