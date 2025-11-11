#[derive(Debug, Clone)]
pub struct OxFieldFrame {
    pub name: String,
    pub value: String,
    pub error: Option<String>,
    pub(crate) default_value: String,

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

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_touched(&self) -> bool {
        self.touched
    }

    pub fn set_touched(&mut self, touched: bool) {
        self.touched = touched;
    }
}
