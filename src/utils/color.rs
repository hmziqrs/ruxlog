use validator::ValidationError;

/// Default colors used across the codebase
pub const DEFAULT_BG_COLOR: &str = "#3b82f6";
pub const DEFAULT_DARK_TEXT: &str = "#111111";
pub const DEFAULT_LIGHT_TEXT: &str = "#ffffff";

/// Normalize a hex color to the form "#rrggbb" (lowercase).
/// Accepts inputs with or without '#'. Returns None if invalid.
pub fn normalize_hex(hex: &str) -> Option<String> {
    let s = hex.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("#{s}").to_lowercase())
}

/// Quick boolean check for hex color validity ("#rrggbb" or "rrggbb").
pub fn is_valid_hex_color(hex: &str) -> bool {
    normalize_hex(hex).is_some()
}

/// Parse "#rrggbb" (or "rrggbb") into RGB tuple.
/// Returns None if `hex` is not a valid color.
pub fn parse_hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Compute a contrasting text color for the given background using YIQ luma.
/// Returns "#111111" for light backgrounds and "#ffffff" for dark backgrounds.
/// Falls back to DEFAULT_DARK_TEXT when parsing fails.
pub fn contrast_text_for_bg(bg_hex: &str) -> String {
    if let Some((r, g, b)) = parse_hex_to_rgb(bg_hex) {
        let yiq = (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000;
        if yiq >= 128 {
            DEFAULT_DARK_TEXT.to_string()
        } else {
            DEFAULT_LIGHT_TEXT.to_string()
        }
    } else {
        DEFAULT_DARK_TEXT.to_string()
    }
}

/// Given a background color and an optional preferred text color, produce a final text color:
/// - If preferred is provided and valid → normalized preferred
/// - Else → computed contrast color from background
pub fn derive_text_color(bg_hex: &str, preferred_text_hex: Option<&str>) -> String {
    if let Some(pref) = preferred_text_hex {
        if let Some(norm) = normalize_hex(pref) {
            return norm;
        }
    }
    contrast_text_for_bg(bg_hex)
}

/// Validator-compatible function: validates a single hex color string input.
pub fn validate_hex_color(s: &str) -> Result<(), ValidationError> {
    if is_valid_hex_color(s) {
        Ok(())
    } else {
        Err(ValidationError::new("hex_color"))
    }
}

/// Validator-compatible function: validates an Option<String> hex color.
/// - None → Ok
/// - Some(invalid) → Err
pub fn validate_optional_hex_color(s: &Option<String>) -> Result<(), ValidationError> {
    match s {
        None => Ok(()),
        Some(v) => validate_hex_color(v),
    }
}

/// Validator-compatible function: validates Option<Option<String>> hex color.
/// - None → Ok      (field omitted)
/// - Some(None) → Ok (explicit null/clear)
/// - Some(Some(invalid)) → Err
pub fn validate_nested_optional_hex_color(
    s: &Option<Option<String>>,
) -> Result<(), ValidationError> {
    match s {
        None => Ok(()),
        Some(None) => Ok(()),
        Some(Some(v)) => validate_hex_color(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize_hex("#ABCDEF").as_deref(), Some("#abcdef"));
        assert_eq!(normalize_hex("abcdef").as_deref(), Some("#abcdef"));
        assert!(normalize_hex("#abcd").is_none());
        assert!(normalize_hex("ghijkl").is_none());
    }

    #[test]
    fn test_parse() {
        assert_eq!(parse_hex_to_rgb("#000000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_to_rgb("#ffffff"), Some((255, 255, 255)));
        assert!(parse_hex_to_rgb("#xyzxyz").is_none());
    }

    #[test]
    fn test_contrast() {
        assert_eq!(contrast_text_for_bg("#ffffff"), DEFAULT_DARK_TEXT);
        assert_eq!(contrast_text_for_bg("#000000"), DEFAULT_LIGHT_TEXT);
    }

    #[test]
    fn test_validators() {
        assert!(validate_hex_color("#aabbcc").is_ok());
        assert!(validate_hex_color("aabbcc").is_ok());
        assert!(validate_hex_color("#aabbc").is_err());

        assert!(validate_optional_hex_color(&None).is_ok());
        assert!(validate_optional_hex_color(&Some("#112233".to_string())).is_ok());
        assert!(validate_optional_hex_color(&Some("bad".to_string())).is_err());

        assert!(validate_nested_optional_hex_color(&None).is_ok());
        assert!(validate_nested_optional_hex_color(&Some(None)).is_ok());
        assert!(validate_nested_optional_hex_color(&Some(Some("#112233".to_string()))).is_ok());
        assert!(validate_nested_optional_hex_color(&Some(Some("bad".to_string()))).is_err());
    }
}
