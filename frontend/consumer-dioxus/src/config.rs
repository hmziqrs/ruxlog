#[derive(Clone, PartialEq)]
pub struct DarkMode(pub bool);

impl DarkMode {
    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }
}

/// Centralized branding configuration
pub struct Brand {
    pub app_name: &'static str,
    pub tagline: &'static str,
    pub copyright_year: &'static str,
    pub author: &'static str,
    pub author_url: &'static str,
    pub repo_url: &'static str,
    pub x_url: &'static str,
    pub dioxus_url: &'static str,
    pub rust_url: &'static str,
}

pub const BRAND: Brand = Brand {
    app_name: "Hmziq.rs Blog",
    tagline: "Thoughts on software and technology",
    copyright_year: "2026",
    author: "hmziqrs",
    author_url: "https://hmziq.rs",
    repo_url: "https://github.com/hmziqrs/ruxlog",
    x_url: "https://x.com/hmziqrs",
    dioxus_url: "https://dioxuslabs.com",
    rust_url: "https://rust-lang.org",
};
