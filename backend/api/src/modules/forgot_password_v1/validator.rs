use serde::{Deserialize, Serialize};
use validator::Validate;

// The verification-code generators (`forgot_password::Entity::generate_code`
// and `email_verification::Entity::generate_code`) both emit 8-char codes, and
// the password floor elsewhere is 12 (auth_v1 / user_v1). These literals must
// stay in sync with those — `validator`'s `length(min/max)` needs integer
// literals, so a shared const can't be used in the attribute. See Phase 3d/3e.
const CODE_LEN: u64 = 8;
const PASSWORD_MIN: u64 = 12;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1GeneratePayload {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1VerifyPayload {
    #[validate(length(min = CODE_LEN, max = CODE_LEN))]
    pub code: String,
    #[validate(email)]
    pub email: String,
}

/// Response from `verify`. On success the emailed code is **consumed** (the
/// stored row is deleted) and a fresh single-use `reset_token` is issued
/// (audit F#9). The client carries this opaque token into `reset`; the original
/// emailed code is no longer valid for anything, so an interceptor who only had
/// the email loses access once the legitimate user verifies.
#[derive(Debug, Deserialize, Serialize)]
pub struct V1VerifyResponse {
    pub reset_token: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1ResetPayload {
    /// REQUIRED (audit V-HIGH-4): the opaque single-use token issued by
    /// `verify`. The password can ONLY be changed through this token — the
    /// legacy path that accepted a raw emailed `code` + `email` directly has
    /// been removed, because `/request`, `/verify` and `/reset` are
    /// independently-reachable routes and an attacker who merely intercepted
    /// the reset email could otherwise call `/reset` directly (skipping
    /// `/verify`) and take over the account. The token binds the user
    /// server-side and is atomically consumed (`GETDEL`) on first use, so the
    /// client no longer sends `code` or `email` here.
    ///
    /// Deserialization fails if this field is absent (no `#[serde(default)]`),
    /// so a tokenless request never reaches the handler.
    #[validate(length(min = 1))]
    pub reset_token: String,
    #[validate(length(min = PASSWORD_MIN))]
    pub password: String,
    #[validate(length(min = PASSWORD_MIN))]
    pub confirm_password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const STRONG_PW: &str = "sup3rstr0ngpw!";

    // V-HIGH-4: a reset request carrying the emailed `code` + `email` (the old
    // takeover path) must NOT deserialize into `V1ResetPayload` — those fields
    // no longer exist, so serde ignores `code` and the required `reset_token`
    // is absent, failing deserialization.
    #[test]
    fn reset_payload_rejects_legacy_code_only_request() {
        let raw = serde_json::json!({
            "code": "12345678",
            "email": "victim@example.com",
            "password": STRONG_PW,
            "confirm_password": STRONG_PW,
        });
        let err = serde_json::from_value::<V1ResetPayload>(raw).unwrap_err();
        // `reset_token` is required; its absence is the failure.
        let msg = err.to_string();
        assert!(
            msg.contains("reset_token"),
            "expected missing-field error for reset_token, got: {msg}"
        );
    }

    // V-HIGH-4: an empty/missing token is rejected. Even though serde rejects an
    // absent field above, a client could send `""`, so the `length(min = 1)`
    // validator must flag it. The handler never sees an empty token.
    #[test]
    fn reset_payload_rejects_empty_token() {
        let payload = V1ResetPayload {
            reset_token: String::new(),
            password: STRONG_PW.to_string(),
            confirm_password: STRONG_PW.to_string(),
        };
        assert!(payload.validate().is_err(), "empty reset_token must fail validation");
    }

    // V-HIGH-4 (positive): a request with a valid (GETDEL-minted) token
    // validates cleanly and can reach the handler. The frontend's
    // `ResetPasswordPayload` sends exactly these three fields.
    #[test]
    fn reset_payload_accepts_valid_token() {
        let payload = V1ResetPayload {
            reset_token: "deadbeefcafebabe".to_string(),
            password: STRONG_PW.to_string(),
            confirm_password: STRONG_PW.to_string(),
        };
        assert!(payload.validate().is_ok(), "a valid token + strong password must validate");
    }

    // V-HIGH-4: confirm_password still gates short/weak passwords independently
    // (kept so the handler's mismatch check is the only other gate).
    #[test]
    fn reset_payload_rejects_short_password() {
        let payload = V1ResetPayload {
            reset_token: "deadbeefcafebabe".to_string(),
            password: "short".to_string(),
            confirm_password: "short".to_string(),
        };
        assert!(payload.validate().is_err(), "short passwords must fail validation");
    }

    // V-HIGH-4: the legacy `code`/`email` fields are silently dropped by serde
    // (deny_unknown_fields is intentionally NOT set, for forward-compat) but
    // cannot substitute for `reset_token`. Confirms a client sending BOTH the
    // old and new shapes still needs a non-empty token.
    #[test]
    fn reset_payload_extra_code_field_does_not_supply_token() {
        let raw = serde_json::json!({
            "code": "12345678",
            "email": "victim@example.com",
            "reset_token": "",
            "password": STRONG_PW,
            "confirm_password": STRONG_PW,
        });
        let payload = serde_json::from_value::<V1ResetPayload>(raw).unwrap();
        assert!(payload.validate().is_err(), "empty reset_token must fail even if code is present");
    }
}
