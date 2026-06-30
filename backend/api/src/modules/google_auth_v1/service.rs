use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use oauth2::basic::{
    BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse,
    BasicTokenType,
};
use oauth2::revocation::StandardRevocableToken;
use oauth2::{
    AuthUrl, Client, ClientId, ClientSecret, RedirectUrl, StandardTokenResponse, TokenUrl,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{error, warn};

use crate::error::{ErrorCode, ErrorResponse};

/// Extra fields Google returns in the token response. We capture the
/// `id_token` so its signature can be verified against Google's JWKS (audit:
/// "id_token trusted without signature verification" — fixed in Phase 3f).
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct IdTokenFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
}
impl oauth2::ExtraTokenFields for IdTokenFields {}

/// OAuth2 client specialized to surface the Google `id_token` from the token
/// exchange. Identical to `BasicClient` except the extra token fields carry
/// `id_token` instead of being empty.
pub type GoogleClient = Client<
    BasicErrorResponse,
    StandardTokenResponse<IdTokenFields, BasicTokenType>,
    BasicTokenType,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
>;

/// The verified claims of a Google `id_token`. Only the identity-bearing fields
/// are decoded here; `aud`/`iss`/`exp` are validated by the JWT library against
/// the raw token, independent of this struct.
///
/// `nonce` (V-LOW-NONCE) is the OIDC nonce we sent in the authorize request and
/// now expect echoed back in the id_token. It is `Option` only because legacy
/// tokens / some flows may omit it; the caller enforces the match when it sent a
/// nonce (see [`verify_google_id_token`]).
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleIdTokenClaims {
    pub sub: String,
    pub email: String,
    #[serde(default)]
    pub email_verified: Option<bool>,
    #[serde(default)]
    pub nonce: Option<String>,
}

/// A single RSA signing key as published in Google's JWKS.
#[derive(Clone, Debug, Deserialize)]
struct GoogleJwkKey {
    kid: Option<String>,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct GoogleJwkSet {
    keys: Vec<GoogleJwkKey>,
}

struct CachedJwks {
    fetched_at: SystemTime,
    keys: Vec<GoogleJwkKey>,
}

const GOOGLE_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";
const GOOGLE_ISSUERS: [&str; 2] = ["https://accounts.google.com", "accounts.google.com"];
const JWKS_TTL: Duration = Duration::from_secs(3600);

/// Process-local JWKS cache. Google rotates its signing keys ~daily; a 1-hour
/// TTL balances freshness against hammering the JWKS endpoint on every login.
/// Each process caches independently — acceptable for an OAuth login path.
static JWKS_CACHE: LazyLock<RwLock<Option<CachedJwks>>> = LazyLock::new(|| RwLock::new(None));

#[allow(clippy::result_large_err)]
pub fn get_google_oauth_client() -> Result<GoogleClient, ErrorResponse> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").map_err(|_| {
        ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("GOOGLE_CLIENT_ID not configured")
    })?;

    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| {
        ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("GOOGLE_CLIENT_SECRET not configured")
    })?;

    let redirect_url = std::env::var("GOOGLE_REDIRECT_URI").map_err(|_| {
        ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("GOOGLE_REDIRECT_URI not configured")
    })?;

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .map_err(|e| {
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Invalid auth URL")
                .with_details(e.to_string())
        })?;

    let token_url =
        TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).map_err(|e| {
            ErrorResponse::new(ErrorCode::InternalServerError)
                .with_message("Invalid token URL")
                .with_details(e.to_string())
        })?;

    let client = GoogleClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).map_err(|e| {
        ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Invalid redirect URI")
            .with_details(e.to_string())
    })?);

    Ok(client)
}

/// Verify a Google `id_token`'s signature and claims against the published JWKS.
///
/// This is defense-in-depth: the user identity is already established via the
/// access token Google issued for the (PKCE-bound) authorization code. Verifying
/// the `id_token` additionally binds the `sub`/`email` cryptographically to
/// Google's signing key and validates `aud` (our client id), `iss`, and `exp`.
///
/// R-5: Google rotates its signing keys roughly daily. If a token arrives
/// signed by a `kid` that is not in our (cached) JWKS, the cache may simply be
/// stale — a brand-new key published within the TTL. Rather than reject (which
/// would lock users out for up to `JWKS_TTL` after each rotation), we force
/// exactly ONE bypass-cache JWKS re-fetch and retry the kid lookup once before
/// rejecting. There is no retry loop: at most one re-fetch per verification.
pub async fn verify_google_id_token(
    id_token: &str,
    expected_aud: &str,
    expected_nonce: Option<&str>,
) -> Result<GoogleIdTokenClaims, ErrorResponse> {
    // First attempt against the (possibly cached) JWKS.
    let keys = fetch_google_jwks().await?;
    match verify_id_token_with_keys_core(
        id_token,
        &keys,
        expected_aud,
        &GOOGLE_ISSUERS,
        expected_nonce,
    ) {
        Ok(claims) => Ok(claims),
        Err(err) => {
            // Only the "unknown signer kid" failure is retry-worthy. Every other
            // failure (bad alg, bad aud/iss/exp, malformed token, bad signature
            // against a *known* key) is deterministic and must NOT trigger a
            // re-fetch — that would just amplify load on a reject path.
            if err.unknown_signer {
                warn!("id_token kid not in cached JWKS; forcing one bypass-cache re-fetch (R-5)");
                let refreshed = fetch_google_jwks_bypass_cache().await?;
                // Single retry against the freshly-fetched key set. If the kid is
                // still absent after a fresh fetch, the token is genuinely
                // untrusted; map the flag-bearing `IdTokenError` into the API
                // `ErrorResponse` the caller expects. NB: call `_core` (which
                // carries the `unknown_signer` flag), NOT the public wrapper
                // `verify_id_token_with_keys` (which strips it — reading
                // `.unknown_signer` off an `ErrorResponse` does not compile).
                verify_id_token_with_keys_core(
                    id_token,
                    &refreshed,
                    expected_aud,
                    &GOOGLE_ISSUERS,
                    expected_nonce,
                )
                .map_err(|e| e.response)
            } else {
                Err(err.response)
            }
        }
    }
}

/// Pure verification core (no network), kept separate so tests can exercise the
/// signature/claims validation with a known key without hitting Google.
///
/// Returns an `IdTokenError` whose `unknown_signer` flag lets the network-aware
/// wrapper (`verify_google_id_token`) decide whether a JWKS re-fetch could
/// possibly help. The public `verify_id_token_with_keys` entry point below maps
/// this into the API's `ErrorResponse`.
// The Err-variant (IdTokenError) is intentionally rich (carries the JWT /
// claims / keys for diagnostics); clippy's size heuristic flags it but the
// design is deliberate.
#[allow(clippy::result_large_err)]
fn verify_id_token_with_keys_core(
    id_token: &str,
    keys: &[GoogleJwkKey],
    expected_aud: &str,
    expected_iss: &[&str],
    expected_nonce: Option<&str>,
) -> Result<GoogleIdTokenClaims, IdTokenError> {
    // Read the JOSE header first to select the signing key by `kid` and to pin
    // the algorithm. Pinning RS256 is mandatory: an "alg confusion" attack would
    // otherwise let a token signed with the public key as an HMAC secret pass.
    let header = decode_header(id_token).map_err(|e| {
        warn!(error = ?e, "Google id_token header decode failed");
        IdTokenError::malformed("Malformed id_token header")
    })?;
    if header.alg != Algorithm::RS256 {
        warn!(alg = ?header.alg, "Rejecting id_token with non-RS256 alg");
        return Err(IdTokenError::malformed("Unsupported id_token algorithm"));
    }

    let signing_key = match header
        .kid
        .as_deref()
        .and_then(|kid| keys.iter().find(|k| k.kid.as_deref() == Some(kid)))
    {
        Some(k) => k,
        // Distinguish "no key for this kid" from other failures so the wrapper
        // can attempt a single JWKS re-fetch (R-5).
        None => {
            warn!("No matching Google signing key for id_token kid");
            return Err(IdTokenError::unknown_signer());
        }
    };

    let decoding_key =
        DecodingKey::from_rsa_components(&signing_key.n, &signing_key.e).map_err(|e| {
            error!(error = ?e, "Failed to build RSA decoding key from Google JWKS");
            IdTokenError::internal("Invalid signing key")
        })?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[expected_aud]);
    validation.set_issuer(expected_iss);
    // `exp` and signature are validated by default.

    let token_data =
        decode::<GoogleIdTokenClaims>(id_token, &decoding_key, &validation).map_err(|e| {
            warn!(error = ?e, "Google id_token verification failed");
            IdTokenError::invalid("Invalid id_token")
        })?;

    // V-LOW-NONCE (OIDC nonce binding): if we sent a nonce in the authorize
    // request, the id_token MUST echo it back verbatim. This binds the token to
    // THIS browser session's authorize request, defeating a token-injection /
    // replay attack where an attacker feeds a victim's id_token into their own
    // session. We validate AFTER signature verification so the nonce claim is
    // cryptographically trusted (it is covered by the signature). A missing or
    // mismatched nonce when one is expected fails closed.
    if let Some(expected) = expected_nonce {
        match &token_data.claims.nonce {
            Some(actual) if actual == expected => { /* bound */ }
            other => {
                warn!(
                    ?other,
                    "id_token nonce missing or mismatched — rejecting login"
                );
                return Err(IdTokenError::invalid("Invalid id_token"));
            }
        }
    }

    Ok(token_data.claims)
}

/// Internal verification outcome that distinguishes the retry-worthy
/// "unknown signer kid" case from terminal failures. Only the network-aware
/// wrapper consumes the `unknown_signer` flag.
#[derive(Debug)]
struct IdTokenError {
    response: ErrorResponse,
    unknown_signer: bool,
}

impl IdTokenError {
    fn unknown_signer() -> Self {
        Self {
            response: ErrorResponse::new(ErrorCode::InvalidToken)
                .with_message("Untrusted id_token signer"),
            unknown_signer: true,
        }
    }

    fn malformed(msg: &str) -> Self {
        Self {
            response: ErrorResponse::new(ErrorCode::InvalidToken).with_message(msg),
            unknown_signer: false,
        }
    }

    fn invalid(msg: &str) -> Self {
        Self {
            response: ErrorResponse::new(ErrorCode::InvalidToken).with_message(msg),
            unknown_signer: false,
        }
    }

    fn internal(msg: &str) -> Self {
        Self {
            response: ErrorResponse::new(ErrorCode::InternalServerError).with_message(msg),
            unknown_signer: false,
        }
    }
}

/// Test-only entry: pure verification against caller-supplied keys, mapping the
/// flag-bearing [`IdTokenError`] into the API [`ErrorResponse`]. The original
/// (pre-R-5) tests supply their own keys and never exercise the JWKS re-fetch,
/// so they go through this thin wrapper; production + the R-5 retry path use
/// [`verify_id_token_with_keys_core`] directly. `#[cfg(test)]` keeps it out of
/// the non-test build (where it would otherwise be dead code).
#[cfg(test)]
#[allow(clippy::result_large_err)] // test-only thin wrapper; ErrorResponse is the domain error type
fn verify_id_token_with_keys(
    id_token: &str,
    keys: &[GoogleJwkKey],
    expected_aud: &str,
    expected_iss: &[&str],
    expected_nonce: Option<&str>,
) -> Result<GoogleIdTokenClaims, ErrorResponse> {
    verify_id_token_with_keys_core(id_token, keys, expected_aud, expected_iss, expected_nonce)
        .map_err(|e| e.response)
}

async fn fetch_google_jwks() -> Result<Vec<GoogleJwkKey>, ErrorResponse> {
    // Fast path: serve from cache if fresh.
    {
        let guard = JWKS_CACHE.read().await;
        if let Some(cached) = guard.as_ref() {
            let fresh = SystemTime::now()
                .duration_since(cached.fetched_at)
                .map(|elapsed| elapsed < JWKS_TTL)
                .unwrap_or(false);
            if fresh {
                return Ok(cached.keys.clone());
            }
        }
    }

    // Cache miss / stale: fetch and refresh the cache.
    fetch_google_jwks_bypass_cache().await
}

/// Fetch Google's JWKS, unconditionally bypassing the cache and overwriting it.
///
/// Used by the R-5 re-fetch path: when a token's `kid` is absent from the
/// cached set (Google rotated a key within our TTL), we must see the freshest
/// set rather than the stale cache entry that just failed us.
async fn fetch_google_jwks_bypass_cache() -> Result<Vec<GoogleJwkKey>, ErrorResponse> {
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        // V-MED-10: bound the JWKS fetch so a wedged Google endpoint can't pin
        // the login handler thread indefinitely (CWE-400/CWE-770).
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(15))
        .pool_idle_timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            error!(error = ?e, "Failed to build JWKS HTTP client");
            ErrorResponse::new(ErrorCode::ExternalServiceError).with_message("JWKS fetch failed")
        })?;

    let resp = http_client.get(GOOGLE_JWKS_URL).send().await.map_err(|e| {
        error!(error = ?e, "Failed to fetch Google JWKS");
        ErrorResponse::new(ErrorCode::ExternalServiceError).with_message("JWKS fetch failed")
    })?;

    let status = resp.status();
    let bytes = resp.bytes().await.map_err(|e| {
        error!(error = ?e, "Failed to read JWKS body");
        ErrorResponse::new(ErrorCode::ExternalServiceError).with_message("JWKS fetch failed")
    })?;

    if !status.is_success() {
        error!(status = %status, "Google JWKS endpoint returned non-2xx");
        return Err(
            ErrorResponse::new(ErrorCode::ExternalServiceError).with_message("JWKS fetch failed")
        );
    }

    let parsed: GoogleJwkSet = serde_json::from_slice(&bytes).map_err(|e| {
        error!(error = ?e, "Failed to parse Google JWKS JSON");
        ErrorResponse::new(ErrorCode::ExternalServiceError).with_message("Malformed JWKS")
    })?;

    let keys = parsed.keys;
    let mut guard = JWKS_CACHE.write().await;
    *guard = Some(CachedJwks {
        fetched_at: SystemTime::now(),
        keys: keys.clone(),
    });

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use jsonwebtoken::{encode, EncodingKey, Header};

    // A throwaway RSA-2048 keypair generated offline (openssl) for unit tests.
    // It is NOT a production secret. Used to sign a fake Google id_token and
    // confirm the verifier accepts a genuine signature and rejects tampering.
    const TEST_RSA_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCi4XBzEby8h8is\n\
W9DK/OoeT4YhoVlnnWaJ1PJos5Mq/ZQpNB7GkpQYew5EUc3WThzpeeEYH6kkykva\n\
Pn9DIu9kG4lV2P/Yu8G5o0HvZvSx2OASisV6XLjZUKd0xJmK2Ss9m/lNpU9pqFF9\n\
ze/TOgN43jbbmNYBD+7WqdOurNmeTyTF5rrd7Ww1IxcsRUhfg+jP5ozkx8xFF7cb\n\
X4Ljnu8oYkZS+oi2Qjcp3IHnIXk+O4J/7QecKJDqjIM0Hv50912Bad/UEtVlXYZ7\n\
ntCCgBGoSifyorwPoI6vEDOSABOt81MfLWFi0H2wG0+FLwbb82x/oF9ZO/KbcNVB\n\
0X6yTVIrAgMBAAECggEAfeOaQwW5h0m3WygXx1wVI1o5hHKtpDzujKdOuIfavkaB\n\
phsHklimKAmsLDfBzNpQ1E+EH14RIENOvx7Aw8YTmp8B1Z1DmWL8xxsckglUJMVH\n\
4mzpVrqlkCkbVE/DkKJrHlIYLOAQ8cvLOF3b97kGB/xQEAgfl3CWG8nkt0QXapfr\n\
CrGZeIofTZRhaBUx9/4ttVjjghJtbIdr4JFGBC+3ZMZ5NbuNGDISMQnOCvUBNj1W\n\
PSBS57ToLZkz80x6zQ+vjTPuo/J3yKx/GaX+H7ECTZfM8JjS7vPTyUJvBrmOyhf7\n\
tFv4XWhRR784GcRENH3kFkMtB5B51UWJB4cBTJVWmQKBgQDVzCd6gnPS1UUfXor9\n\
RTKBeRJBC9ZoZUyKPPry4Pjb8sYBmnW4HWRL1MlC0ycuQX7ZFTPdc+mONf6RsLjd\n\
5MHykr1632tio+8IWVwum39TDmq5uQAXeSpaqAMOAkH7QRQNPymK9zEMqFibFPHm\n\
8wXYWnyE3lvWXoeWs896rTLhzwKBgQDDCEyhijYak4kga5TFslTPwxz36S/R0vZc\n\
DtRdM0EdDqJ62RfPpz/spAikiHtuhpT3HVqZZomeQ8M1sbYpaP4EjI7J8fmVBBJ5\n\
y/kT5uA6Jy1I38V8zxKe4jtdMzs1olMJJP+k542tpclOXjbgyBoTgUXmleqzk3bG\n\
YB4Pqpps5QKBgBGMLxVUDbOZQ5IejWPaQRn1WPUzxoZNAio6dRJoOqS62VuaVN0m\n\
tGuw7E/qysV2JLYmklozwFCmx90nVxUHSI/jUV/7ZHH1KJJT20gMBThI76OMtqA2\n\
lq5YKeAFeWro3X901rEMNt9mFdessWoWOj2Wt6+kHH+MxK4u1fGos4trAoGAR9Xn\n\
u9xXf0R2Tp2xh3ve50ObiOi391X37gJ8T/PP+O7qA8uwjIiy7+ufT1MB+7zQY5DJ\n\
TRVKfSPCZCWXzfrhDTXkZhedcTi1wWzSynTQhDrn4B6j9AuldSYo7XQwS9oFMaoS\n\
C2BKe/pDgn0LQ5IQoLyNzZfMgeY/6mN+zxBsns0CgYEAiDb8ssmD+oi7RPFp7Wcz\n\
gyKHV5IglzjfQJ8zl4NkTZrV+ivXkhfVyIFYAT+PEh5ZgevUD01DSJTKvF7UiqaS\n\
Fx8SA361H10YXKqf8gUpNfjl/ixAJzV+ROeAUqCzK+liGuXkFCXbs7Ey0OPQfu5h\n\
JX3Efq+lIpLs6bXvFyKzuRc=\n-----END PRIVATE KEY-----";

    // JWK public components for the key above (base64urlurl, no padding).
    const TEST_N: &str = "ouFwcxG8vIfIrFvQyvzqHk-GIaFZZ51midTyaLOTKv2UKTQexpKUGHsORFHN1k4c6XnhGB-pJMpL2j5_QyLvZBuJVdj_2LvBuaNB72b0sdjgEorFely42VCndMSZitkrPZv5TaVPaahRfc3v0zoDeN4225jWAQ_u1qnTrqzZnk8kxea63e1sNSMXLEVIX4Poz-aM5MfMRRe3G1-C457vKGJGUvqItkI3KdyB5yF5PjuCf-0HnCiQ6oyDNB7-dPddgWnf1BLVZV2Ge57QgoARqEon8qK8D6COrxAzkgATrfNTHy1hYtB9sBtPhS8G2_Nsf6BfWTvym3DVQdF-sk1SKw";
    const TEST_E: &str = "AQAB";
    const TEST_KID: &str = "test-key-1";
    const TEST_AUD: &str = "test-client-id.apps.googleusercontent.com";

    fn test_keys() -> Vec<GoogleJwkKey> {
        vec![GoogleJwkKey {
            kid: Some(TEST_KID.to_string()),
            n: TEST_N.to_string(),
            e: TEST_E.to_string(),
        }]
    }

    fn sign_token(claims: &serde_json::Value, kid: Option<&str>) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = kid.map(|s| s.to_string());
        let key = EncodingKey::from_rsa_pem(TEST_RSA_PEM.as_bytes()).unwrap();
        encode(&header, claims, &key).unwrap()
    }

    fn valid_claims() -> serde_json::Value {
        let exp = (Utc::now() + chrono::Duration::hours(1)).timestamp();
        serde_json::json!({
            "sub": "google-sub-123",
            "email": "user@example.com",
            "email_verified": true,
            "aud": TEST_AUD,
            "iss": "https://accounts.google.com",
            "exp": exp,
            "iat": Utc::now().timestamp(),
        })
    }

    #[test]
    fn id_token_verifies_with_matching_signature_and_claims() {
        let token = sign_token(&valid_claims(), Some(TEST_KID));
        let claims =
            verify_id_token_with_keys(&token, &test_keys(), TEST_AUD, &GOOGLE_ISSUERS, None)
                .unwrap();
        assert_eq!(claims.sub, "google-sub-123");
        assert_eq!(claims.email, "user@example.com");
        assert_eq!(claims.email_verified, Some(true));
    }

    #[test]
    fn id_token_rejected_when_tampered() {
        let token = sign_token(&valid_claims(), Some(TEST_KID));
        // Corrupt the signature segment (after the last '.') so the signature no
        // longer matches. This mirrors a real forge: an attacker who edits the
        // claims but cannot re-sign with Google's key.
        let parts: Vec<&str> = token.rsplitn(2, '.').collect();
        // parts = [signature, header.payload]
        let mut sig_bytes = parts[0].as_bytes().to_vec();
        // Flip the last byte to a different (still base64url-valid) char.
        if let Some(b) = sig_bytes.last_mut() {
            *b = if *b == b'A' { b'B' } else { b'A' };
        }
        let sig = String::from_utf8(sig_bytes).unwrap();
        let tampered = format!("{}.{}", parts[1], sig);
        let result =
            verify_id_token_with_keys(&tampered, &test_keys(), TEST_AUD, &GOOGLE_ISSUERS, None);
        assert!(result.is_err(), "tampered token must be rejected");
    }

    #[test]
    fn id_token_rejected_for_wrong_audience() {
        let token = sign_token(&valid_claims(), Some(TEST_KID));
        let result = verify_id_token_with_keys(
            &token,
            &test_keys(),
            "other-client-id",
            &GOOGLE_ISSUERS,
            None,
        );
        assert!(result.is_err(), "wrong-audience token must be rejected");
    }

    #[test]
    fn id_token_rejected_for_unknown_kid() {
        let token = sign_token(&valid_claims(), Some("not-a-known-kid"));
        let result =
            verify_id_token_with_keys(&token, &test_keys(), TEST_AUD, &GOOGLE_ISSUERS, None);
        assert!(
            result.is_err(),
            "token signed by unknown kid must be rejected"
        );
    }

    #[test]
    fn id_token_rejected_when_expired() {
        let mut claims = valid_claims();
        let past = (Utc::now() - chrono::Duration::hours(2)).timestamp();
        claims["exp"] = serde_json::json!(past);
        let token = sign_token(&claims, Some(TEST_KID));
        let result =
            verify_id_token_with_keys(&token, &test_keys(), TEST_AUD, &GOOGLE_ISSUERS, None);
        assert!(result.is_err(), "expired token must be rejected");
    }

    // R-5: the network-aware wrapper retries a JWKS fetch ONLY for the
    // "unknown signer kid" outcome. That decision hinges on the pure core
    // flagging `unknown_signer` distinctly from every terminal failure. This
    // test pins that predicate directly, without a live network call.
    #[test]
    fn unknown_kid_is_the_only_retry_worthy_failure() {
        // Unknown kid -> retry-worthy.
        let token = sign_token(&valid_claims(), Some("not-a-known-kid"));
        let err =
            verify_id_token_with_keys_core(&token, &test_keys(), TEST_AUD, &GOOGLE_ISSUERS, None)
                .expect_err("unknown kid must error");
        assert!(
            err.unknown_signer,
            "unknown kid must be flagged retry-worthy"
        );

        // Tampered signature against a KNOWN key -> terminal (a re-fetch cannot
        // help; the key is already present and the signature is wrong).
        let token = sign_token(&valid_claims(), Some(TEST_KID));
        let parts: Vec<&str> = token.rsplitn(2, '.').collect();
        let mut sig_bytes = parts[0].as_bytes().to_vec();
        if let Some(b) = sig_bytes.last_mut() {
            *b = if *b == b'A' { b'B' } else { b'A' };
        }
        let tampered = format!("{}.{}", parts[1], String::from_utf8(sig_bytes).unwrap());
        let err = verify_id_token_with_keys_core(
            &tampered,
            &test_keys(),
            TEST_AUD,
            &GOOGLE_ISSUERS,
            None,
        )
        .expect_err("tampered token must error");
        assert!(
            !err.unknown_signer,
            "tampered token must NOT trigger a re-fetch"
        );

        // Wrong audience against a KNOWN key -> terminal.
        let token = sign_token(&valid_claims(), Some(TEST_KID));
        let err = verify_id_token_with_keys_core(
            &token,
            &test_keys(),
            "other-client-id",
            &GOOGLE_ISSUERS,
            None,
        )
        .expect_err("wrong-audience token must error");
        assert!(
            !err.unknown_signer,
            "wrong-audience token must NOT trigger a re-fetch"
        );
    }

    // R-5: simulate the exact retry sequence the wrapper runs when Google
    // rotates a key within our cache TTL. The first attempt uses a "stale" key
    // set that lacks the token's kid; the re-fetched set contains it. The
    // second attempt must succeed — i.e. a valid token is no longer rejected
    // just because the cache was stale. This exercises the orchestration
    // shape (one retry, then succeed) without hitting the network.
    #[test]
    fn stale_jwks_then_refresh_recovers_rotated_kid() {
        let token = sign_token(&valid_claims(), Some(TEST_KID));

        // "Stale" cached set: empty -> kid lookup misses.
        let stale_keys: Vec<GoogleJwkKey> = vec![];
        let first =
            verify_id_token_with_keys_core(&token, &stale_keys, TEST_AUD, &GOOGLE_ISSUERS, None);
        assert!(
            first
                .as_ref()
                .err()
                .map(|e| e.unknown_signer)
                .unwrap_or(false),
            "stale JWKS must fail with the retry-worthy unknown_signer flag"
        );

        // "Refreshed" set: contains the rotated kid -> the single retry succeeds.
        let refreshed_keys = test_keys();
        let second = verify_id_token_with_keys_core(
            &token,
            &refreshed_keys,
            TEST_AUD,
            &GOOGLE_ISSUERS,
            None,
        )
        .expect("refreshed JWKS must verify the rotated-kid token");
        assert_eq!(second.sub, "google-sub-123");
    }

    // V-LOW-NONCE: when an expected nonce is supplied, a token echoing that
    // exact nonce verifies, while a token with a different (or absent) nonce is
    // rejected — binding the id_token to the authorize request that initiated
    // the flow. jsonwebtoken has no built-in nonce check, so this pins our
    // manual post-signature claim comparison.
    #[test]
    fn id_token_nonce_must_match_when_expected() {
        let mut claims = valid_claims();
        claims["nonce"] = serde_json::json!("the-nonce-we-sent");
        let token = sign_token(&claims, Some(TEST_KID));

        // Matching nonce -> accepted.
        let ok = verify_id_token_with_keys(
            &token,
            &test_keys(),
            TEST_AUD,
            &GOOGLE_ISSUERS,
            Some("the-nonce-we-sent"),
        )
        .expect("token echoing the expected nonce must verify");
        assert_eq!(ok.nonce.as_deref(), Some("the-nonce-we-sent"));

        // Wrong nonce -> rejected.
        let err = verify_id_token_with_keys(
            &token,
            &test_keys(),
            TEST_AUD,
            &GOOGLE_ISSUERS,
            Some("a-different-nonce"),
        );
        assert!(err.is_err(), "mismatched nonce must be rejected");

        // No nonce expected (legacy / non-nonce flow) -> still accepted.
        let ok_none =
            verify_id_token_with_keys(&token, &test_keys(), TEST_AUD, &GOOGLE_ISSUERS, None)
                .expect("no-nonce-expected flow must still verify");
        assert_eq!(ok_none.sub, "google-sub-123");

        // Token MISSING the nonce claim while one is expected -> rejected.
        let token_without_nonce = sign_token(&valid_claims(), Some(TEST_KID));
        let err = verify_id_token_with_keys(
            &token_without_nonce,
            &test_keys(),
            TEST_AUD,
            &GOOGLE_ISSUERS,
            Some("the-nonce-we-sent"),
        );
        assert!(
            err.is_err(),
            "token missing the nonce claim must be rejected when a nonce is expected"
        );
    }
}
