use regex::Regex;

/// Extract first paragraph from HTML content for use as description
pub fn extract_first_paragraph(html_content: &str) -> Option<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html_content);
    let p_selector = Selector::parse("p").ok()?;

    // Find first non-empty paragraph
    for element in document.select(&p_selector) {
        let text = element.text().collect::<String>();
        let trimmed = text.trim();
        if !trimmed.is_empty() && trimmed.len() > 20 {
            // Clean up and truncate
            return Some(clean_text(trimmed));
        }
    }

    None
}

/// Clean text by removing extra whitespace and HTML entities
pub fn clean_text(text: &str) -> String {
    // Remove extra whitespace
    let re = Regex::new(r"\s+").unwrap();
    let cleaned = re.replace_all(text, " ");

    // Decode common HTML entities
    cleaned
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_string()
}

/// Generate excerpt from content with character limit
pub fn generate_excerpt(content: &str, max_chars: usize) -> String {
    let cleaned = clean_text(content);

    if cleaned.len() <= max_chars {
        return cleaned;
    }

    // Find last sentence boundary before max_chars
    let truncated = &cleaned[..max_chars];
    if let Some(pos) = truncated.rfind(|c| c == '.' || c == '!' || c == '?') {
        cleaned[..=pos].to_string()
    } else if let Some(pos) = truncated.rfind(' ') {
        format!("{}...", &cleaned[..pos])
    } else {
        format!("{}...", truncated)
    }
}

/// Validate image URL is absolute or convert to absolute
pub fn ensure_absolute_url(url: &str, base_url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        let path = if url.starts_with('/') { url } else { &format!("/{}", url) };
        format!("{}{}", base_url.trim_end_matches('/'), path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "Hello   world  &nbsp; &amp; test";
        let result = clean_text(input);
        assert_eq!(result, "Hello world & test");
    }

    #[test]
    fn test_generate_excerpt_short() {
        let content = "Short content.";
        let result = generate_excerpt(content, 100);
        assert_eq!(result, content);
    }

    #[test]
    fn test_generate_excerpt_long() {
        let content = "This is the first sentence. This is the second sentence. This is a very long third sentence that should be truncated.";
        let result = generate_excerpt(content, 60);
        assert!(result.ends_with('.'));
        assert!(result.len() <= 60);
    }

    #[test]
    fn test_ensure_absolute_url_already_absolute() {
        let url = "https://example.com/image.jpg";
        let result = ensure_absolute_url(url, "https://base.com");
        assert_eq!(result, url);
    }

    #[test]
    fn test_ensure_absolute_url_relative() {
        let url = "/images/test.jpg";
        let result = ensure_absolute_url(url, "https://base.com");
        assert_eq!(result, "https://base.com/images/test.jpg");
    }

    #[test]
    fn test_ensure_absolute_url_no_leading_slash() {
        let url = "images/test.jpg";
        let result = ensure_absolute_url(url, "https://base.com");
        assert_eq!(result, "https://base.com/images/test.jpg");
    }
}
