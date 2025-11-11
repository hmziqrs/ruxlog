use std::collections::HashMap;
use validator::Validate;

pub trait OxFormModel: Validate + Clone + PartialEq {
    fn to_map(&self) -> HashMap<String, String>;
    fn update_field(&mut self, name: String, value: &str);
}
