use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::db::sea_models::user::{NewUser, UserRole};

fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{1,}$").unwrap();
    if email_regex.is_match(email) {
        Ok(())
    } else {
        Err(ValidationError::new("Invalid email format"))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct V1LoginPayload {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1RegisterPayload {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(email, custom(function = "validate_email"))]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

impl V1RegisterPayload {
    pub fn into_new_user(self) -> NewUser {
        NewUser {
            name: self.name,
            email: self.email,
            password: self.password,
            role: UserRole::User,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1TwoFAVerifyPayload {
    #[validate(length(min = 6, max = 12))]
    pub code: String,
    #[validate(length(min = 6, max = 64))]
    pub backup_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1TwoFADisablePayload {
    #[validate(length(min = 6, max = 64))]
    pub code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1TerminateSessionPath {
    pub id: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_email ────────────────────────────────────────────────────

    #[test]
    fn validate_email_valid_addresses() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user@domain.org").is_ok());
        assert!(validate_email("a+b@c.co").is_ok());
        assert!(validate_email("name@sub.domain.com").is_ok());
        assert!(validate_email("X@Y.Z").is_ok());
    }

    #[test]
    fn validate_email_invalid_no_at() {
        assert!(validate_email("userexample.com").is_err());
    }

    #[test]
    fn validate_email_invalid_no_domain() {
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn validate_email_invalid_no_tld() {
        assert!(validate_email("user@domain").is_err());
    }

    #[test]
    fn validate_email_invalid_empty() {
        assert!(validate_email("").is_err());
    }

    #[test]
    fn validate_email_invalid_double_at() {
        assert!(validate_email("user@@domain.com").is_err());
    }

    #[test]
    fn validate_email_invalid_spaces() {
        assert!(validate_email("user @domain.com").is_err());
        assert!(validate_email("user@ domain.com").is_err());
    }

    // ── V1RegisterPayload::into_new_user ──────────────────────────────────

    #[test]
    fn into_new_user_maps_fields() {
        let payload = V1RegisterPayload {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "s3cret".to_string(),
        };
        let new_user = payload.into_new_user();
        assert_eq!(new_user.name, "Alice");
        assert_eq!(new_user.email, "alice@example.com");
        assert_eq!(new_user.password, "s3cret");
        assert_eq!(new_user.role, UserRole::User);
    }

    // ── V1RegisterPayload validation ──────────────────────────────────────

    #[test]
    fn register_payload_valid() {
        let payload = V1RegisterPayload {
            name: "Bob".to_string(),
            email: "bob@test.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn register_payload_empty_name_fails() {
        let payload = V1RegisterPayload {
            name: "".to_string(),
            email: "bob@test.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(payload.validate().is_err());
    }

    #[test]
    fn register_payload_empty_password_fails() {
        let payload = V1RegisterPayload {
            name: "Bob".to_string(),
            email: "bob@test.com".to_string(),
            password: "".to_string(),
        };
        assert!(payload.validate().is_err());
    }

    #[test]
    fn register_payload_invalid_email_fails() {
        let payload = V1RegisterPayload {
            name: "Bob".to_string(),
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(payload.validate().is_err());
    }

    // ── V1LoginPayload validation ─────────────────────────────────────────

    #[test]
    fn login_payload_valid() {
        let payload = V1LoginPayload {
            email: "user@host.com".to_string(),
            password: "pass".to_string(),
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn login_payload_empty_password_fails() {
        let payload = V1LoginPayload {
            email: "user@host.com".to_string(),
            password: "".to_string(),
        };
        assert!(payload.validate().is_err());
    }
}
