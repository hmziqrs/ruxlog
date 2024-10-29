use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::db::models::user::{NewUser, UserRole};

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
