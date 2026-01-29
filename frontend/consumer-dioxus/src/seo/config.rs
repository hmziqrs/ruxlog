use crate::config::BRAND;

pub struct SeoConfig {
    pub site_name: &'static str,
    pub site_tagline: &'static str,
    pub site_url: &'static str,
    pub consumer_url: &'static str,
    pub default_image: &'static str,
    pub default_description: &'static str,
    pub twitter_handle: Option<&'static str>,
    pub locale: &'static str,
}

pub const SEO_CONFIG: SeoConfig = SeoConfig {
    site_name: BRAND.app_name,
    site_tagline: BRAND.tagline,
    site_url: crate::env::APP_API_URL,
    consumer_url: get_consumer_url(),
    default_image: "/assets/og-default.png",
    default_description: "Modern blog covering Rust, Dioxus, web development, and technology",
    twitter_handle: Some("@hmziqrs"),
    locale: "en_US",
};

/// Get consumer URL from environment or default
const fn get_consumer_url() -> &'static str {
    match std::option_env!("CONSUMER_SITE_URL") {
        Some(url) => url,
        None => "http://localhost:1108",
    }
}

/// Format page title with site name
/// Format: "Page Title | Site Name" or "Site Name - Tagline" if title is empty
pub fn format_title(page_title: &str) -> String {
    if page_title.is_empty() {
        format!("{} - {}", SEO_CONFIG.site_name, SEO_CONFIG.site_tagline)
    } else {
        format!("{} | {}", page_title, SEO_CONFIG.site_name)
    }
}

/// Generate canonical URL for a route path
pub fn canonical_url(path: &str) -> String {
    let path = if path.starts_with('/') { path } else { &format!("/{}", path) };
    format!("{}{}", SEO_CONFIG.consumer_url, path)
}

/// Truncate description to SEO-optimal length (150-160 characters)
pub fn truncate_description(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    // Find last space before max_len to avoid cutting mid-word
    let truncated = &text[..max_len];
    if let Some(pos) = truncated.rfind(' ') {
        format!("{}...", &text[..pos])
    } else {
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_title_empty() {
        let result = format_title("");
        assert!(result.contains(SEO_CONFIG.site_name));
        assert!(result.contains(SEO_CONFIG.site_tagline));
    }

    #[test]
    fn test_format_title_with_page() {
        let result = format_title("About");
        assert_eq!(result, "About | Hmziq.rs Blog");
    }

    #[test]
    fn test_canonical_url() {
        let result = canonical_url("/posts/123");
        assert!(result.ends_with("/posts/123"));
    }

    #[test]
    fn test_canonical_url_without_leading_slash() {
        let result = canonical_url("posts/123");
        assert!(result.ends_with("/posts/123"));
    }

    #[test]
    fn test_truncate_description_short() {
        let text = "Short description";
        let result = truncate_description(text, 160);
        assert_eq!(result, text);
    }

    #[test]
    fn test_truncate_description_long() {
        let text = "This is a very long description that needs to be truncated to fit within the SEO-optimal length of 150-160 characters for meta descriptions to ensure good search engine results";
        let result = truncate_description(text, 160);
        assert!(result.len() <= 163); // 160 + "..."
        assert!(result.ends_with("..."));
    }
}
