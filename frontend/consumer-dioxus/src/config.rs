#[derive(Clone, PartialEq)]
pub struct DarkMode(pub bool);

impl DarkMode {
    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }
}
