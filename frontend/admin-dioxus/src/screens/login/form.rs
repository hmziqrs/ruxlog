use std::collections::HashMap;

use dioxus::prelude::*;

use validator::Validate;

use crate::hooks::{OxForm, OxFormModel};

#[derive(Debug, Validate, Clone, PartialEq)]
pub struct LoginForm {
    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,

    #[validate(length(min = 4, message = "Password must be at least 4 characters"))]
    pub password: String,

    /// TOTP code for the second step of 2FA-at-login (F#4/F#7/F#16). Populated
    /// only when the server returned a `totp_required` response.
    #[validate(length(min = 6, max = 6, message = "Code must be 6 digits"))]
    pub totp_code: String,
}

impl LoginForm {
    #[allow(dead_code)]
    pub fn new() -> Self {
        LoginForm {
            email: String::new(),
            password: String::new(),
            totp_code: String::new(),
        }
    }

    pub fn dev() -> Self {
        LoginForm {
            email: String::from("emmie_quisquam@gmail.com"),
            password: String::from("emmie_quisquam@gmail.com"),
            totp_code: String::new(),
            // email: String::from("hmzi@gmail.com"),
            // password: String::from("hmzi@gmail.com"),
            // email: String::from("hmzi@gmail.rs"),
            // password: String::from("hello"),
        }
    }
}

impl OxFormModel for LoginForm {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("email".to_string(), self.email.clone());
        map.insert("password".to_string(), self.password.clone());
        map.insert("totp_code".to_string(), self.totp_code.clone());
        map
    }

    fn update_field(&mut self, name: String, value: &str) {
        match name.as_str() {
            "email" => self.email = value.to_string(),
            "password" => self.password = value.to_string(),
            "totp_code" => self.totp_code = value.to_string(),
            _ => {}
        }
    }
}

pub fn use_login_form(initial_state: LoginForm) -> Signal<OxForm<LoginForm>> {
    let form_slice: Signal<OxForm<LoginForm>> = use_signal(|| OxForm::new(initial_state));

    form_slice
}
