use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::sea_models::media::{MediaQuery, MediaReference};
use crate::utils::SortParam;

// ── M-7: MIME + extension allowlist ────────────────────────────────────────
//
// There is no server-side file-type gate, so a caller can upload
// `image/svg+xml` (or any extension) and the raw bytes — `<script>` and all —
// are stored to S3 and served verbatim: stored XSS. The `image` crate is
// compiled with only jpeg/png/webp/tiff decoders, so an svg is never decoded
// and its bytes pass straight through.
//
// These helpers define the single source of truth for accepted image types.
// `validate_upload` is called from the controller before any bytes are hashed
// or persisted; `allowlisted_extension` is used by `infer_extension` so a
// client-supplied extension that is not on the list is dropped rather than
// baked into the stored object key.

/// Accepted MIME types (lowercase, without parameters). Anything else — most
/// importantly `image/svg+xml` — is rejected at upload time.
pub const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/tiff",
];

/// Accepted file extensions (lowercase, no leading dot). A client-supplied
/// extension that is not in this set is stripped from the stored key.
pub const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "tiff"];

/// Returns true if `mime` (case-insensitive, parameters stripped) is on the
/// allowlist.
pub fn is_allowed_mime(mime: &str) -> bool {
    let normalized = mime
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    ALLOWED_MIME_TYPES
        .iter()
        .any(|allowed| *allowed == normalized)
}

/// Returns the lowercased extension if it is on the allowlist, else `None`.
/// Input may include a leading dot; it is stripped before comparison.
pub fn allowlisted_extension(ext: &str) -> Option<String> {
    let normalized = ext.trim().trim_start_matches('.').to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }
    if ALLOWED_EXTENSIONS
        .iter()
        .any(|allowed| *allowed == normalized)
    {
        Some(normalized)
    } else {
        None
    }
}

/// Resolve an upload to its final (mime, extension) pair, enforcing the
/// allowlist. Returns `Err(message)` for disallowed types (SVG, octet-stream,
/// anything off the list). When the declared MIME is allowed but no extension
/// could be inferred from the filename, the extension is derived from the MIME.
pub fn validate_upload(
    mime: Option<&str>,
    filename: Option<&str>,
) -> Result<(String, String), String> {
    let normalized_mime = mime
        .map(|m| {
            m.split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase()
        })
        .filter(|m| !m.is_empty());

    let normalized_mime = match normalized_mime {
        Some(m) if is_allowed_mime(&m) => m,
        // No declared MIME, or a declared MIME that is not on the list: try to
        // infer from the filename extension, then validate that.
        _ => {
            let ext = filename
                .and_then(|n| n.rsplit_once('.'))
                .map(|(_, e)| e.trim().to_ascii_lowercase())
                .filter(|e| !e.is_empty());

            match ext.as_deref() {
                Some(e) if ALLOWED_EXTENSIONS.contains(&e) => mime_for_extension(e).to_string(),
                _ => {
                    return Err(format!(
                        "Unsupported file type. Allowed: {}",
                        ALLOWED_MIME_TYPES.join(", ")
                    ));
                }
            }
        }
    };

    // Extension: prefer an allowlisted filename extension; else derive from MIME.
    let extension = filename
        .and_then(|n| n.rsplit_once('.'))
        .map(|(_, e)| e.trim().to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .and_then(|e| {
            if ALLOWED_EXTENSIONS.iter().any(|a| *a == e) {
                Some(e)
            } else {
                None
            }
        })
        .unwrap_or_else(|| extension_for_mime(&normalized_mime).to_string());

    Ok((normalized_mime, extension))
}

fn mime_for_extension(ext: &str) -> &'static str {
    match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "tiff" => "image/tiff",
        _ => "application/octet-stream",
    }
}

fn extension_for_mime(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/tiff" => "tiff",
        _ => "bin",
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, Validate)]
pub struct MediaUploadMetadata {
    pub reference_type: Option<MediaReference>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl MediaUploadMetadata {
    pub fn apply_field(&mut self, name: &str, value: &str) -> Result<(), String> {
        match name {
            "reference_type" => {
                if value.trim().is_empty() {
                    self.reference_type = None;
                } else {
                    self.reference_type = Some(MediaReference::from_str(value.trim())?);
                }
            }
            "width" => {
                if value.trim().is_empty() {
                    self.width = None;
                } else {
                    self.width = Some(
                        value
                            .trim()
                            .parse::<i32>()
                            .map_err(|_| format!("Invalid width: {}", value.trim()))?,
                    );
                }
            }
            "height" => {
                if value.trim().is_empty() {
                    self.height = None;
                } else {
                    self.height = Some(
                        value
                            .trim()
                            .parse::<i32>()
                            .map_err(|_| format!("Invalid height: {}", value.trim()))?,
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1MediaListQuery {
    pub page: Option<u64>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>, // [{ field, order }]
    pub reference_type: Option<MediaReference>,
    pub uploader_id: Option<i32>,
    pub mime_type: Option<String>,
    pub extension: Option<String>,
    // Optional created_at/updated_at range filters (ISO8601)
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}

impl V1MediaListQuery {
    /// Build the DB query, resolving the effective `uploader_id` filter.
    ///
    /// M-5: a non-privileged caller is always scoped to their own media. Only
    /// when `is_privileged` is true (moderator/staff or above) do we honor an
    /// explicit `uploader_id`; otherwise the caller's id overrides whatever
    /// they supplied, so a regular author cannot enumerate another user's
    /// object keys, buckets, public URLs, or usage.
    pub fn into_query(self, caller_id: i32, is_privileged: bool) -> MediaQuery {
        let uploader_id = if is_privileged {
            self.uploader_id
        } else {
            Some(caller_id)
        };

        MediaQuery {
            page: self.page,
            search: self.search,
            sorts: self.sorts,
            reference_type: self.reference_type,
            uploader_id,
            mime_type: self.mime_type,
            extension: self.extension,
            created_at_gt: self.created_at_gt,
            created_at_lt: self.created_at_lt,
            updated_at_gt: self.updated_at_gt,
            updated_at_lt: self.updated_at_lt,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1MediaUsageQuery {
    #[validate(length(min = 1, message = "media_ids must contain at least one id"))]
    pub media_ids: Vec<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── MediaUploadMetadata::apply_field ──────────────────────────────────

    #[test]
    fn apply_field_reference_type_valid() {
        let mut meta = MediaUploadMetadata::default();
        assert!(meta.reference_type.is_none());

        meta.apply_field("reference_type", "category").unwrap();
        assert_eq!(meta.reference_type, Some(MediaReference::Category));

        meta.apply_field("reference_type", "user").unwrap();
        assert_eq!(meta.reference_type, Some(MediaReference::User));

        meta.apply_field("reference_type", "post").unwrap();
        assert_eq!(meta.reference_type, Some(MediaReference::Post));
    }

    #[test]
    fn apply_field_reference_type_empty_clears() {
        let mut meta = MediaUploadMetadata {
            reference_type: Some(MediaReference::Category),
            ..Default::default()
        };

        meta.apply_field("reference_type", "  ").unwrap();
        assert!(meta.reference_type.is_none());

        meta.apply_field("reference_type", "").unwrap();
        assert!(meta.reference_type.is_none());
    }

    #[test]
    fn apply_field_reference_type_invalid() {
        let mut meta = MediaUploadMetadata::default();
        let result = meta.apply_field("reference_type", "invalid_type");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid media reference type"));
    }

    #[test]
    fn apply_field_width_valid() {
        let mut meta = MediaUploadMetadata::default();
        assert!(meta.width.is_none());

        meta.apply_field("width", "1920").unwrap();
        assert_eq!(meta.width, Some(1920));

        meta.apply_field("width", "0").unwrap();
        assert_eq!(meta.width, Some(0));

        meta.apply_field("width", "-100").unwrap();
        assert_eq!(meta.width, Some(-100));
    }

    #[test]
    fn apply_field_width_empty_clears() {
        let mut meta = MediaUploadMetadata {
            width: Some(800),
            ..Default::default()
        };

        meta.apply_field("width", "").unwrap();
        assert!(meta.width.is_none());
    }

    #[test]
    fn apply_field_width_invalid() {
        let mut meta = MediaUploadMetadata::default();
        let result = meta.apply_field("width", "abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid width"));
    }

    #[test]
    fn apply_field_height_valid() {
        let mut meta = MediaUploadMetadata::default();
        assert!(meta.height.is_none());

        meta.apply_field("height", "1080").unwrap();
        assert_eq!(meta.height, Some(1080));
    }

    #[test]
    fn apply_field_height_empty_clears() {
        let mut meta = MediaUploadMetadata {
            height: Some(600),
            ..Default::default()
        };

        meta.apply_field("height", "  ").unwrap();
        assert!(meta.height.is_none());
    }

    #[test]
    fn apply_field_height_invalid() {
        let mut meta = MediaUploadMetadata::default();
        let result = meta.apply_field("height", "not_a_number");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid height"));
    }

    #[test]
    fn apply_field_unknown_field_ignored() {
        let mut meta = MediaUploadMetadata::default();
        // Unknown fields are silently ignored -- no error
        let result = meta.apply_field("unknown_field", "some_value");
        assert!(result.is_ok());
        assert!(meta.reference_type.is_none());
        assert!(meta.width.is_none());
        assert!(meta.height.is_none());
    }

    #[test]
    fn apply_field_whitespace_trimmed() {
        let mut meta = MediaUploadMetadata::default();

        meta.apply_field("width", "  500  ").unwrap();
        assert_eq!(meta.width, Some(500));

        meta.apply_field("reference_type", "  category  ").unwrap();
        assert_eq!(meta.reference_type, Some(MediaReference::Category));
    }

    // ── M-7: MIME / extension allowlist ───────────────────────────────────

    #[test]
    fn is_allowed_mime_accepts_known_image_types() {
        for mime in ALLOWED_MIME_TYPES {
            assert!(is_allowed_mime(mime), "{} should be allowed", mime);
        }
    }

    #[test]
    fn is_allowed_mime_rejects_svg() {
        assert!(!is_allowed_mime("image/svg+xml"));
    }

    #[test]
    fn is_allowed_mime_is_case_insensitive_and_strips_parameters() {
        assert!(is_allowed_mime("IMAGE/PNG"));
        assert!(is_allowed_mime("image/jpeg; charset=utf-8"));
        assert!(is_allowed_mime("  image/webp  "));
    }

    #[test]
    fn is_allowed_mime_rejects_non_image() {
        assert!(!is_allowed_mime("application/octet-stream"));
        assert!(!is_allowed_mime("text/html"));
        assert!(!is_allowed_mime(""));
    }

    #[test]
    fn allowlisted_extension_normalizes_and_validates() {
        assert_eq!(allowlisted_extension("png"), Some("png".to_string()));
        assert_eq!(allowlisted_extension(".JPG"), Some("jpg".to_string()));
        assert_eq!(allowlisted_extension("  .tiff "), Some("tiff".to_string()));
    }

    #[test]
    fn allowlisted_extension_rejects_svg_and_arbitrary() {
        assert_eq!(allowlisted_extension("svg"), None);
        assert_eq!(allowlisted_extension("html"), None);
        assert_eq!(allowlisted_extension("exe"), None);
        assert_eq!(allowlisted_extension(""), None);
    }

    #[test]
    fn validate_upload_accepts_allowlisted_png() {
        let (mime, ext) = validate_upload(Some("image/png"), Some("photo.png")).unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(ext, "png");
    }

    #[test]
    fn validate_upload_rejects_svg_explicit_mime() {
        let result = validate_upload(Some("image/svg+xml"), Some("x.svg"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported file type"));
    }

    #[test]
    fn validate_upload_rejects_svg_via_extension_only() {
        // No declared MIME; extension alone is not on the allowlist.
        let result = validate_upload(None, Some("payload.svg"));
        assert!(result.is_err());
    }

    #[test]
    fn validate_upload_rejects_arbitrary_content_type() {
        let result = validate_upload(Some("text/html"), Some("a.html"));
        assert!(result.is_err());
    }

    #[test]
    fn validate_upload_infers_mime_from_allowlisted_extension() {
        // No Content-Type header, but an allowlisted extension is trusted.
        let (mime, ext) = validate_upload(None, Some("scan.JPG")).unwrap();
        assert_eq!(mime, "image/jpeg");
        assert_eq!(ext, "jpg");
    }

    #[test]
    fn validate_upload_strips_untrusted_extension_but_keeps_allowed_mime() {
        // Declared MIME is allowed; filename extension is junk — extension is
        // derived from the MIME rather than trusted.
        let (mime, ext) = validate_upload(Some("image/png"), Some("trick.svg")).unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(ext, "png");
    }

    #[test]
    fn validate_upload_rejects_when_neither_mime_nor_ext_allowed() {
        let result = validate_upload(Some("application/octet-stream"), Some("data.bin"));
        assert!(result.is_err());
    }

    // ── M-5: uploader_id scoping ──────────────────────────────────────────

    #[test]
    fn into_query_scopes_non_privileged_to_caller() {
        let query = V1MediaListQuery {
            page: None,
            search: None,
            sorts: None,
            reference_type: None,
            uploader_id: Some(999), // attempts to read another user's media
            mime_type: None,
            extension: None,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
        };

        let media_query = query.into_query(7, false);
        // Non-privileged caller is forced to their own id; the supplied 999 is
        // discarded.
        assert_eq!(media_query.uploader_id, Some(7));
    }

    #[test]
    fn into_query_honors_explicit_uploader_for_privileged() {
        let query = V1MediaListQuery {
            page: None,
            search: None,
            sorts: None,
            reference_type: None,
            uploader_id: Some(999),
            mime_type: None,
            extension: None,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
        };

        let media_query = query.into_query(7, true);
        // Privileged caller may target any uploader id.
        assert_eq!(media_query.uploader_id, Some(999));
    }

    #[test]
    fn into_query_privileged_with_no_filter_lists_all() {
        let query = V1MediaListQuery {
            page: None,
            search: None,
            sorts: None,
            reference_type: None,
            uploader_id: None,
            mime_type: None,
            extension: None,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
        };

        let media_query = query.into_query(7, true);
        assert_eq!(media_query.uploader_id, None);
    }
}
