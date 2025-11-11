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
}

impl LoginForm {
    #[allow(dead_code)]
    pub fn new() -> Self {
        LoginForm {
            email: String::new(),
            password: String::new(),
        }
    }

    pub fn dev() -> Self {
        LoginForm {
            email: String::from("emmie_quisquam@gmail.com"),
            password: String::from("emmie_quisquam@gmail.com"),
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
        map
    }

    fn update_field(&mut self, name: String, value: &str) {
        match name.as_str() {
            "email" => self.email = value.to_string(),
            "password" => self.password = value.to_string(),
            _ => {}
        }
    }
}

pub fn use_login_form(initial_state: LoginForm) -> Signal<OxForm<LoginForm>> {
    let form_slice: Signal<OxForm<LoginForm>> = use_signal(|| OxForm::new(initial_state));

    form_slice
}
