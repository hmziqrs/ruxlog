use dioxus::prelude::*;
use std::collections::HashMap;
use validator::{Validate, ValidationError};

use crate::hooks::{OxForm, OxFormModel};
use crate::store::{TagsAddPayload, TagsEditPayload};
use crate::utils::colors::get_contrast_yiq;

#[derive(Debug, Validate, Clone, PartialEq)]
pub struct TagForm {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(length(min = 1, message = "Slug is required"))]
    #[validate(custom(function = "validate_slug"))]
    pub slug: String,

    pub description: String,
    // Hex color like #3b82f6
    pub color: String,
    // Optional override for text color when custom_text_color is true
    pub text_color: String,
    // Whether to use custom text color instead of auto-contrast
    pub custom_text_color: bool,
    // Visibility: whether the tag is publicly visible
    pub active: bool,
}

fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    let regex = regex::Regex::new(r"^[a-z0-9-_]+$").unwrap();
    if !regex.is_match(slug) {
        return Err(ValidationError::new(
            "Slug can only contain lowercase letters, numbers, hyphens and underscores",
        ));
    }
    Ok(())
}

impl TagForm {
    pub fn new() -> Self {
        TagForm {
            name: String::new(),
            slug: String::new(),
            description: String::new(),
            color: "#3b82f6".to_string(),
            // default text color based on auto contrast of default bg
            text_color: get_contrast_yiq("#3b82f6").to_string(),
            custom_text_color: false,
            active: true,
        }
    }

    pub fn sanitize_slug(text: &str) -> String {
        let text = text.to_lowercase();
        let text = regex::Regex::new(r"[^\w\s-]")
            .unwrap()
            .replace_all(&text, "")
            .to_string();
        let text = regex::Regex::new(r"\s+")
            .unwrap()
            .replace_all(&text, "-")
            .to_string();
        let text = regex::Regex::new(r"-+")
            .unwrap()
            .replace_all(&text, "-")
            .to_string();
        let text = regex::Regex::new(r"^-+|-+$")
            .unwrap()
            .replace_all(&text, "")
            .to_string();
        text
    }

    // Compute the final text color respecting user choice or auto-contrast
    pub fn effective_text_color(&self) -> String {
        if self.custom_text_color && !self.text_color.trim().is_empty() {
            self.text_color.clone()
        } else {
            get_contrast_yiq(&self.color).to_string()
        }
    }

    // Convert the form to the backend add payload contract
    pub fn to_add_payload(&self) -> TagsAddPayload {
        let description = if self.description.trim().is_empty() {
            None
        } else {
            Some(self.description.clone())
        };
        let color = if self.color.trim().is_empty() {
            None
        } else {
            Some(self.color.clone())
        };
        let text_color = Some(self.effective_text_color());
        let is_active = Some(self.active);

        TagsAddPayload {
            name: self.name.clone(),
            slug: self.slug.clone(),
            description,
            color,
            text_color,
            is_active,
        }
    }

    // Convert the form to the backend edit payload contract
    pub fn to_edit_payload(&self) -> TagsEditPayload {
        let description = if self.description.trim().is_empty() {
            None
        } else {
            Some(self.description.clone())
        };
        let color = if self.color.trim().is_empty() {
            None
        } else {
            Some(self.color.clone())
        };
        let text_color = Some(self.effective_text_color());
        let is_active = Some(self.active);

        TagsEditPayload {
            name: Some(self.name.clone()),
            slug: Some(self.slug.clone()),
            description,
            color,
            text_color,
            is_active,
        }
    }
}

impl OxFormModel for TagForm {
    fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("name".to_string(), self.name.clone());
        map.insert("slug".to_string(), self.slug.clone());
        map.insert("description".to_string(), self.description.clone());
        map.insert("color".to_string(), self.color.clone());
        map.insert("text_color".to_string(), self.text_color.clone());
        map.insert(
            "custom_text_color".to_string(),
            if self.custom_text_color {
                "true".to_string()
            } else {
                "false".to_string()
            },
        );
        map.insert(
            "active".to_string(),
            if self.active {
                "true".to_string()
            } else {
                "false".to_string()
            },
        );
        map
    }

    fn update_field(&mut self, name: String, value: &str) {
        match name.as_str() {
            "name" => self.name = value.to_string(),
            "slug" => self.slug = value.to_string(),
            "description" => self.description = value.to_string(),
            "color" => self.color = value.to_string(),
            "text_color" => self.text_color = value.to_string(),
            "custom_text_color" => {
                let v = value.trim().to_lowercase();
                self.custom_text_color = matches!(v.as_str(), "true" | "1" | "yes" | "on");
            }
            "active" => {
                let v = value.trim().to_lowercase();
                self.active = matches!(v.as_str(), "true" | "1" | "yes" | "on");
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseTagForm {
    pub form: Signal<OxForm<TagForm>>,
}

pub fn use_tag_form(initial_state: TagForm) -> UseTagForm {
    let form_signal: Signal<OxForm<TagForm>> = use_signal(move || OxForm::new(initial_state));

    UseTagForm { form: form_signal }
}
