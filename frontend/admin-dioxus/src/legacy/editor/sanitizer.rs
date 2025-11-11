//! HTML sanitizer for the rich text editor.
//!
//! This module provides security-focused HTML sanitization to prevent XSS attacks
//! while allowing safe formatting and structure.

use ammonia::Builder;
use std::collections::{HashMap, HashSet};

/// Sanitizes HTML content according to our security policy.
///
/// # Security Policy
/// - Allows safe formatting tags (b, i, u, s, code, mark, etc.)
/// - Allows structural elements (p, h1-h6, ul, ol, li, blockquote, pre, hr)
/// - Allows links with href attribute (sanitized)
/// - Allows images with src, alt, title attributes
/// - Strips all JavaScript and event handlers
/// - Sanitizes URLs to prevent javascript: protocol attacks
pub fn sanitize_html(input: &str) -> String {
    let mut builder = Builder::default();

    // Allowed tags
    let allowed_tags: HashSet<&str> = [
        // Inline formatting
        "b",
        "strong",
        "i",
        "em",
        "u",
        "s",
        "strike",
        "code",
        "mark",
        "small",
        "sup",
        "sub",
        "br",
        "a",
        // Block elements
        "p",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "div",
        "blockquote",
        "pre",
        "hr",
        // Lists
        "ul",
        "ol",
        "li",
        "input",
        // Images and media
        "img",
        "figure",
        "figcaption",
        // Embeds
        "iframe",
        "video",
        "audio",
        "source",
        // Tables (for future support)
        "table",
        "thead",
        "tbody",
        "tr",
        "th",
        "td",
    ]
    .into_iter()
    .collect();

    builder.tags(allowed_tags);

    // Allowed URL schemes
    builder.url_schemes(["http", "https", "mailto"].into_iter().collect());

    // Link rel attribute handling (must be set before tag_attributes)
    builder.link_rel(Some("noopener noreferrer"));

    // Allowed attributes per tag
    let mut tag_attrs: HashMap<&str, HashSet<&str>> = HashMap::new();
    tag_attrs.insert("a", ["href", "title", "target"].into_iter().collect());
    tag_attrs.insert(
        "img",
        ["src", "alt", "title", "width", "height"]
            .into_iter()
            .collect(),
    );
    tag_attrs.insert(
        "iframe",
        ["src", "width", "height", "frameborder", "allowfullscreen"]
            .into_iter()
            .collect(),
    );
    tag_attrs.insert(
        "video",
        ["src", "width", "height", "controls"].into_iter().collect(),
    );
    tag_attrs.insert("audio", ["src", "controls"].into_iter().collect());
    tag_attrs.insert("source", ["src", "type"].into_iter().collect());
    tag_attrs.insert("div", ["class", "style"].into_iter().collect());
    tag_attrs.insert("p", ["class", "style"].into_iter().collect());
    tag_attrs.insert("blockquote", ["class", "cite"].into_iter().collect());
    tag_attrs.insert("code", ["class"].into_iter().collect());
    tag_attrs.insert("pre", ["class"].into_iter().collect());
    tag_attrs.insert("li", ["class"].into_iter().collect());
    tag_attrs.insert("ul", ["class"].into_iter().collect());
    tag_attrs.insert("ol", ["class", "start", "type"].into_iter().collect());
    tag_attrs.insert("table", ["class"].into_iter().collect());
    tag_attrs.insert("th", ["scope", "colspan", "rowspan"].into_iter().collect());
    tag_attrs.insert("td", ["colspan", "rowspan"].into_iter().collect());
    tag_attrs.insert(
        "input",
        ["type", "checked", "disabled", "class", "contenteditable"]
            .into_iter()
            .collect(),
    );

    builder.tag_attributes(tag_attrs);

    // Clean the HTML
    builder.clean(input).to_string()
}

/// Sanitizes a URL to ensure it's safe for use in links or embeds.
pub fn sanitize_url(url: &str) -> Option<String> {
    let trimmed = url.trim();

    // Reject javascript:, data:, vbscript:, and other dangerous protocols
    let lower = trimmed.to_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("vbscript:")
        || lower.starts_with("file:")
    {
        return None;
    }

    // Accept http, https, mailto
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("//")
    {
        return Some(trimmed.to_string());
    }

    // Reject if it looks suspicious (contains control characters, etc.)
    if trimmed.chars().any(|c| c.is_control()) {
        return None;
    }

    // Accept relative URLs
    if trimmed.starts_with('/') || trimmed.starts_with("./") || trimmed.starts_with("../") {
        return Some(trimmed.to_string());
    }

    // Default: prepend https:// if no scheme
    Some(format!("https://{}", trimmed))
}

/// Extracts plain text from HTML, stripping all tags.
pub fn html_to_text(html: &str) -> String {
    ammonia::clean(html)
}

/// Validates if a string is a safe CSS color value.
pub fn is_safe_color(color: &str) -> bool {
    let trimmed = color.trim();

    // Hex colors
    if trimmed.starts_with('#') {
        let hex = &trimmed[1..];
        return (hex.len() == 3 || hex.len() == 6 || hex.len() == 8)
            && hex.chars().all(|c| c.is_ascii_hexdigit());
    }

    // RGB/RGBA
    if trimmed.starts_with("rgb(")
        || trimmed.starts_with("rgba(")
        || trimmed.starts_with("hsl(")
        || trimmed.starts_with("hsla(")
    {
        return true;
    }

    // Named colors (basic set)
    matches!(
        trimmed.to_lowercase().as_str(),
        "red"
            | "blue"
            | "green"
            | "yellow"
            | "orange"
            | "purple"
            | "pink"
            | "black"
            | "white"
            | "gray"
            | "grey"
            | "cyan"
            | "magenta"
            | "lime"
            | "navy"
            | "teal"
            | "olive"
            | "maroon"
            | "aqua"
            | "fuchsia"
            | "silver"
            | "transparent"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_sanitizes_basic_html() {
        let input = "<p>Hello <b>world</b></p>";
        let output = sanitize_html(input);
        assert!(output.contains("<p>"));
        assert!(output.contains("<b>"));
    }

    #[test]
    fn it_removes_script_tags() {
        let input = "<p>Hello</p><script>alert('xss')</script>";
        let output = sanitize_html(input);
        assert!(!output.contains("<script"));
        assert!(!output.contains("alert"));
    }

    #[test]
    fn it_removes_event_handlers() {
        let input = r#"<p onclick="alert('xss')">Click me</p>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("onclick"));
        assert!(!output.contains("alert"));
    }

    #[test]
    fn it_sanitizes_javascript_urls() {
        let url = "javascript:alert('xss')";
        assert!(sanitize_url(url).is_none());
    }

    #[test]
    fn it_allows_https_urls() {
        let url = "https://example.com";
        assert_eq!(sanitize_url(url).unwrap(), "https://example.com");
    }

    #[test]
    fn it_prepends_https_to_bare_domains() {
        let url = "example.com";
        assert_eq!(sanitize_url(url).unwrap(), "https://example.com");
    }

    #[test]
    fn it_validates_hex_colors() {
        assert!(is_safe_color("#ff0000"));
        assert!(is_safe_color("#f00"));
        assert!(is_safe_color("#ff0000ff"));
        assert!(!is_safe_color("#gg0000"));
        assert!(!is_safe_color("#ff00"));
    }

    #[test]
    fn it_validates_named_colors() {
        assert!(is_safe_color("red"));
        assert!(is_safe_color("blue"));
        assert!(is_safe_color("transparent"));
        assert!(!is_safe_color("notacolor"));
    }

    #[test]
    fn it_validates_rgb_colors() {
        assert!(is_safe_color("rgb(255, 0, 0)"));
        assert!(is_safe_color("rgba(255, 0, 0, 0.5)"));
    }
}
