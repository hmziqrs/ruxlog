use std::collections::HashMap;

use dioxus::prelude::*;
use validator::Validate;

use crate::hooks::{OxForm, OxFormModel};

#[derive(Debug, Validate, Clone, PartialEq)]
pub struct RegisterForm {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(email(message = "Please enter a valid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[validate(length(min = 8, message = "Please confirm your password"))]
    pub confirm_password: String,
}

impl RegisterForm {
    #[allow(dead_code)]
    pub fn new() -> Self {
        RegisterForm {
            name: String::new(),
            email: String::new(),
            password: String::new(),
            confirm_password: String::new(),
        }
    }

    pub fn dev() -> Self {
        RegisterForm {
            name: String::from(""),
            email: String::from(""),
            password: String::from(""),
            confirm_password: String::from(""),
        }
    }
}

impl OxFormModel for RegisterForm {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("email".to_string(), self.email.clone());
        map.insert("password".to_string(), self.password.clone());
        map.insert("confirm_password".to_string(), self.confirm_password.clone());
        map
    }

    fn update_field(&mut self, name: String, value: &str) {
        match name.as_str() {
            "name" => self.name = value.to_string(),
            "email" => self.email = value.to_string(),
            "password" => self.password = value.to_string(),
            "confirm_password" => self.confirm_password = value.to_string(),
            _ => {}
        }
    }
}

pub fn use_register_form(initial_state: RegisterForm) -> Signal<OxForm<RegisterForm>> {
    let form_slice: Signal<OxForm<RegisterForm>> = use_signal(|| OxForm::new(initial_state));

    form_slice
}
