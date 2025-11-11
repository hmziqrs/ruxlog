use dioxus::prelude::*;
use std::collections::HashMap;
use validator::Validate;

use crate::hooks::{OxForm, OxFormModel};
use crate::store::{Media, UserRole, UsersAddPayload, UsersEditPayload};

#[derive(Debug, Validate, Clone, PartialEq)]
pub struct UserForm {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    pub role: String,

    pub is_verified: bool,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: Option<String>,

    pub confirm_password: Option<String>,

    pub avatar_id: Option<i32>,

    pub avatar_blob_url: Option<String>,

    /// Existing avatar from server (used when editing)
    #[validate(skip)]
    pub existing_avatar: Option<Media>,

    pub is_update: bool,
}

impl UserForm {
    pub fn new() -> Self {
        UserForm {
            name: String::new(),
            email: String::new(),
            role: UserRole::User.to_string(),
            is_verified: false,
            password: Some(String::new()),
            confirm_password: Some(String::new()),
            avatar_id: None,
            avatar_blob_url: None,
            existing_avatar: None,
            is_update: false,
        }
    }

    pub fn to_add_payload(&self) -> UsersAddPayload {
        UsersAddPayload {
            name: self.name.clone(),
            email: self.email.clone(),
            password: self.password.clone().unwrap_or_default(),
            role: self.role.clone(),
            avatar_id: self.avatar_id,
            is_verified: self.is_verified,
        }
    }

    pub fn to_edit_payload(&self) -> UsersEditPayload {
        UsersEditPayload {
            name: Some(self.name.clone()),
            email: Some(self.email.clone()),
            avatar_id: self.avatar_id,
            password: self.password.clone(),
            is_verified: Some(self.is_verified),
            role: Some(self.role.clone()),
        }
    }
}

impl OxFormModel for UserForm {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("email".to_string(), self.email.clone());
        map.insert("role".to_string(), self.role.to_string());
        map.insert("is_verified".to_string(), self.is_verified.to_string());

        if let Some(password) = &self.password {
            map.insert("password".to_string(), password.clone());
        } else {
            map.insert("password".to_string(), String::new());
        }

        if let Some(confirm_password) = &self.confirm_password {
            map.insert("confirm_password".to_string(), confirm_password.clone());
        } else {
            map.insert("confirm_password".to_string(), String::new());
        }

        if let Some(avatar_id) = self.avatar_id {
            map.insert("avatar_id".to_string(), avatar_id.to_string());
        }

        if let Some(avatar_blob_url) = &self.avatar_blob_url {
            map.insert("avatar_blob_url".to_string(), avatar_blob_url.clone());
        }

        map
    }

    fn update_field(&mut self, name: String, value: &str) {
        match name.as_str() {
            "name" => self.name = value.to_string(),
            "email" => self.email = value.to_string(),
            "role" => self.role = value.to_string(),
            "is_verified" => {
                let v = value.trim().to_lowercase();
                self.is_verified = matches!(v.as_str(), "true" | "1" | "yes" | "on");
            }
            "password" => {
                if self.is_update && value.is_empty() {
                    self.password = None;
                } else {
                    self.password = Some(value.to_string());
                }
            }
            "confirm_password" => {
                if self.is_update && value.is_empty() {
                    self.confirm_password = None;
                } else {
                    self.confirm_password = Some(value.to_string());
                }
            }
            "avatar_id" => {
                self.avatar_id = value.parse::<i32>().ok();
            }
            "avatar_blob_url" => {
                self.avatar_blob_url = if value.is_empty() {
                    None
                } else {
                    Some(value.to_string())
                };
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseUserForm {
    pub form: Signal<OxForm<UserForm>>,
}

pub fn use_user_form(initial_state: UserForm) -> UseUserForm {
    let form_signal: Signal<OxForm<UserForm>> = use_signal(move || OxForm::new(initial_state));

    UseUserForm { form: form_signal }
}
