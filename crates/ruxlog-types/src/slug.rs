use regex::Regex;

pub fn sanitize_slug(text: &str) -> String {
    let text = text.to_lowercase();
    let text = Regex::new(r"[^\w\s-]")
        .unwrap()
        .replace_all(&text, "")
        .to_string();
    let text = Regex::new(r"\s+")
        .unwrap()
        .replace_all(&text, "-")
        .to_string();
    let text = Regex::new(r"-+")
        .unwrap()
        .replace_all(&text, "-")
        .to_string();
    let text = Regex::new(r"^-+|-+$")
        .unwrap()
        .replace_all(&text, "")
        .to_string();
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slug() {
        assert_eq!(sanitize_slug("Hello World"), "hello-world");
    }

    #[test]
    fn special_chars_removed() {
        assert_eq!(sanitize_slug("Hello, World! (2024)"), "hello-world-2024");
    }

    #[test]
    fn multiple_dashes_collapsed() {
        assert_eq!(sanitize_slug("a---b"), "a-b");
    }

    #[test]
    fn leading_trailing_dashes_trimmed() {
        assert_eq!(sanitize_slug("--hello--"), "hello");
    }

    #[test]
    fn empty_string() {
        assert_eq!(sanitize_slug(""), "");
    }
}
