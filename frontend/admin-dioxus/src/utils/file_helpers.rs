use web_sys::File;

/// Format file size in bytes to human-readable format
/// Examples: 1024 -> "1.0 KB", 1536000 -> "1.5 MB"
pub fn format_file_size(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes.abs() as f64;
    let exp = (bytes_f.ln() / 1024_f64.ln()).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);

    let size = bytes_f / 1024_f64.powi(exp as i32);
    let unit = UNITS[exp];

    if exp == 0 {
        format!("{} {}", bytes, unit)
    } else {
        format!("{:.1} {}", size, unit)
    }
}

/// Get icon class name based on mime type
/// Returns lucide icon class for different file types
pub fn get_file_icon(mime_type: &str) -> &'static str {
    if mime_type.starts_with("image/") {
        "lucide-image"
    } else if mime_type.starts_with("video/") {
        "lucide-video"
    } else if mime_type.starts_with("audio/") {
        "lucide-music"
    } else if mime_type.contains("pdf") {
        "lucide-file-text"
    } else if mime_type.contains("zip") || mime_type.contains("rar") || mime_type.contains("tar") {
        "lucide-archive"
    } else if mime_type.contains("word") || mime_type.contains("document") {
        "lucide-file-text"
    } else if mime_type.contains("sheet") || mime_type.contains("excel") {
        "lucide-sheet"
    } else {
        "lucide-file"
    }
}

/// Validate if a file type is allowed
/// allowed_types: list of mime type prefixes (e.g., ["image/", "video/"])
pub fn validate_file_type(file: &File, allowed_types: &[&str]) -> bool {
    if allowed_types.is_empty() {
        return true; // No restrictions
    }

    let file_type = file.type_();
    allowed_types
        .iter()
        .any(|allowed| file_type.starts_with(allowed))
}

/// Get file extension from filename
pub fn get_file_extension(filename: &str) -> Option<String> {
    filename.rsplit('.').next().and_then(|ext| {
        if ext.len() < filename.len() {
            Some(ext.to_lowercase())
        } else {
            None
        }
    })
}

/// Check if file is an image based on mime type
pub fn is_image(mime_type: &str) -> bool {
    mime_type.starts_with("image/")
}

/// Check if file is a video based on mime type
pub fn is_video(mime_type: &str) -> bool {
    mime_type.starts_with("video/")
}

/// Get shortened filename for display (truncate middle if too long)
pub fn truncate_filename(filename: &str, max_length: usize) -> String {
    if filename.len() <= max_length {
        return filename.to_string();
    }

    let extension = get_file_extension(filename);
    let ext_len = extension.as_ref().map(|e| e.len() + 1).unwrap_or(0);
    let available = max_length.saturating_sub(ext_len + 3); // 3 for "..."

    if available < 5 {
        // Too short, just truncate
        return format!("{}...", &filename[..max_length.saturating_sub(3)]);
    }

    let start_len = available / 2;
    let end_len = available - start_len;

    match extension {
        Some(ext) => {
            let name_without_ext = &filename[..filename.len() - ext.len() - 1];
            format!(
                "{}...{}.{}",
                &name_without_ext[..start_len.min(name_without_ext.len())],
                &name_without_ext[name_without_ext.len().saturating_sub(end_len)..],
                ext
            )
        }
        None => {
            format!(
                "{}...{}",
                &filename[..start_len],
                &filename[filename.len() - end_len..]
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1572864), "1.5 MB");
    }

    #[test]
    fn test_get_file_icon() {
        assert_eq!(get_file_icon("image/png"), "lucide-image");
        assert_eq!(get_file_icon("video/mp4"), "lucide-video");
        assert_eq!(get_file_icon("audio/mpeg"), "lucide-music");
        assert_eq!(get_file_icon("application/pdf"), "lucide-file-text");
        assert_eq!(get_file_icon("application/zip"), "lucide-archive");
        assert_eq!(get_file_icon("unknown/type"), "lucide-file");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension("test.png"), Some("png".to_string()));
        assert_eq!(get_file_extension("archive.tar.gz"), Some("gz".to_string()));
        assert_eq!(get_file_extension("noextension"), None);
        assert_eq!(get_file_extension("file.PDF"), Some("pdf".to_string()));
    }

    #[test]
    fn test_truncate_filename() {
        assert_eq!(truncate_filename("short.png", 20), "short.png");
        assert_eq!(
            truncate_filename("very_long_filename_here.png", 20),
            "very_l...here.png"
        );
        assert_eq!(
            truncate_filename("noextension_but_very_long", 15),
            "noext...ry_long"
        );
    }
}
