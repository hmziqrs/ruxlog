use std::collections::HashMap;

use dioxus::logger::tracing;
use validator::{Validate, ValidationErrors};

#[derive(Debug, Clone)]
pub struct OxFieldFrame {
    pub name: String,
    pub value: String,
    pub error: Option<String>,
    default_value: String,

    focused: bool,
    touched: bool,
    dirty: bool,
}

impl OxFieldFrame {
    pub fn new(name: String, value: String) -> Self {
        OxFieldFrame {
            name,
            value: value.clone(),
            default_value: value,

            error: None,
            focused: false,
            touched: false,
            dirty: false,
        }
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }
}

pub trait OxFormModel: Validate + Clone + PartialEq {
    fn to_map(&self) -> HashMap<String, String>;
    fn update_field(&mut self, name: String, value: &str);
}

#[derive(Debug, Clone)]
pub struct OxForm<T: OxFormModel> {
    pub data: T,
    pub fields: HashMap<String, OxFieldFrame>,
    pub has_errors: bool,
    pub active_field: Option<String>,

    pub submit_count: u32,
}

impl<T: OxFormModel> OxForm<T> {
    pub fn new(data: T) -> Self {
        let initial_map = data.to_map();
        let mut fields = HashMap::new();

        for (name, value) in &initial_map {
            fields.insert(name.clone(), OxFieldFrame::new(name.clone(), value.clone()));
        }

        OxForm {
            data,
            fields,
            active_field: None,
            has_errors: false,
            submit_count: 0,
        }
    }

    pub fn update_field(&mut self, name: &str, value: String) {
        // Update the field slice
        if let Some(field) = self.fields.get_mut(name) {
            field.value = value.clone();
            field.dirty = field.value != field.default_value;
        }

        // Update the underlying data model
        self.data.update_field(name.to_string(), &value);

        // Validate after update
        self.validate();
    }

    pub fn is_dirty(&self) -> bool {
        self.fields.values().any(|field| field.dirty)
    }

    pub fn is_valid(&self) -> bool {
        !self.has_errors
    }

    pub fn get_field(&self, name: &str) -> Option<&OxFieldFrame> {
        self.fields.get(name)
    }

    pub fn focus_field(&mut self, name: &str) {
        self.active_field = Some(name.to_string());

        for (field_name, field) in self.fields.iter_mut() {
            if field_name == name {
                field.touched = true;
                field.focused = true;
            } else {
                field.focused = false;
            }
        }
    }

    pub fn blur_field(&mut self, name: &str) {
        self.active_field = None;

        self.validate();

        if let Some(field) = self.fields.get_mut(name) {
            field.focused = false;
        }
    }

    pub fn validate(&mut self) -> bool {
        if self.submit_count == 0 {
            return false;
        }
        for field in self.fields.values_mut() {
            field.set_error(None);
        }

        // Validate using validator crate
        match self.data.validate() {
            Ok(()) => {
                self.has_errors = false;
                true
            }
            Err(errors) => {
                self.apply_validation_errors(errors);
                self.has_errors = true;
                false
            }
        }
    }

    fn apply_validation_errors(&mut self, errors: ValidationErrors) {
        for (field_name, field_errors) in errors.field_errors() {
            if let Some(field) = self.fields.get_mut(field_name.as_ref()) {
                if let Some(first_error) = field_errors.first() {
                    // Get the error message
                    let message = match &first_error.message {
                        Some(msg) => msg.to_string(),
                        None => format!("Invalid {}", field_name),
                    };
                    field.set_error(Some(message));
                }
            }
        }
    }

    pub fn on_submit(&mut self, callback: impl Fn(T)) {
        self.submit_count += 1;

        tracing::info!("pub fn on_submit(&");

        if self.validate() {
            callback(self.data.clone());
        }
    }
}
