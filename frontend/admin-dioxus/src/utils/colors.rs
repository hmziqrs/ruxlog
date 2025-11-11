// Shared color utility functions used across the app

/// Convert a hex color string like "#fff" or "#112233" into (r,g,b)
pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim().trim_start_matches('#');
    let full = match hex.len() {
        3 => {
            let mut s = String::with_capacity(6);
            for c in hex.chars() {
                s.push(c);
                s.push(c);
            }
            s
        }
        6 => hex.to_string(),
        _ => return None,
    };
    u32::from_str_radix(&full, 16).ok().map(|val| {
        let r = ((val >> 16) & 0xFF) as u8;
        let g = ((val >> 8) & 0xFF) as u8;
        let b = (val & 0xFF) as u8;
        (r, g, b)
    })
}

/// Return a high-contrast text color ("#111111" or "#ffffff") for a given background hex color
pub fn get_contrast_yiq(hex: &str) -> &'static str {
    if let Some((r, g, b)) = hex_to_rgb(hex) {
        let yiq = (r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000;
        if yiq >= 128 {
            "#111111"
        } else {
            "#ffffff"
        }
    } else {
        "#111111"
    }
}
