use axum::{routing::get, Router};

use crate::AppState;

pub mod controller {
    use axum::{
        extract::{Query, State},
        http::{header, HeaderValue, StatusCode},
        response::{IntoResponse, Response},
    };
    use axum_macros::debug_handler;
    use chrono::Utc;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
    use serde::Deserialize;

    use crate::{
        db::sea_models::post::{self, Column as PostColumn, Entity as PostEntity, PostStatus},
        error::ErrorResponse,
        AppState,
    };

    #[derive(Debug, Deserialize)]
    pub struct FeedQuery {
        pub limit: Option<u64>,
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\"', "&quot;")
            .replace('\'', "&apos;")
    }

    // Extract a short plain-text summary from Editor.js-style JSON
    fn content_to_summary(value: &serde_json::Value, max_len: usize) -> String {
        let mut out = String::new();
        if let Some(blocks) = value.get("blocks").and_then(|b| b.as_array()) {
            for b in blocks {
                if let Some(typ) = b.get("type").and_then(|t| t.as_str()) {
                    let text = match typ {
                        "paragraph" | "header" | "quote" => b
                            .get("data")
                            .and_then(|d| d.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string(),
                        "alert" => b
                            .get("data")
                            .and_then(|d| d.get("message"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string(),
                        "checklist" => {
                            if let Some(items) = b
                                .get("data")
                                .and_then(|d| d.get("items"))
                                .and_then(|i| i.as_array())
                            {
                                let texts: Vec<&str> = items
                                    .iter()
                                    .filter_map(|it| it.get("text").and_then(|t| t.as_str()))
                                    .collect();
                                texts.join(", ")
                            } else {
                                String::new()
                            }
                        }
                        "code" => b
                            .get("data")
                            .and_then(|d| d.get("code"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("")
                            .to_string(),
                        _ => String::new(),
                    };
                    if !text.is_empty() {
                        if !out.is_empty() {
                            out.push(' ');
                        }
                        out.push_str(&text);
                        if out.len() >= max_len {
                            break;
                        }
                    }
                }
            }
        }
        if out.is_empty() {
            // fallback to the entire JSON as string, trimmed
            out = value.to_string();
        }
        out.chars().take(max_len).collect()
    }

    #[allow(clippy::result_large_err)]
    fn build_xml_response(
        content_type: &'static str,
        xml: String,
    ) -> Result<Response, ErrorResponse> {
        let mut builder = axum::http::Response::builder().status(StatusCode::OK);
        builder = builder.header(header::CONTENT_TYPE, content_type);
        builder = builder.header(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=300, s-maxage=300"),
        );

        match builder.body(axum::body::Body::from(xml)) {
            Ok(resp) => Ok(resp),
            Err(_) => Err(crate::error::ErrorResponse::new(
                crate::error::ErrorCode::InternalServerError,
            )),
        }
    }

    async fn fetch_latest_posts(
        state: &AppState,
        limit: u64,
    ) -> Result<Vec<post::Model>, ErrorResponse> {
        let posts = PostEntity::find()
            .filter(PostColumn::Status.eq(PostStatus::Published))
            .order_by_desc(PostColumn::PublishedAt)
            .order_by_desc(PostColumn::UpdatedAt)
            .limit(limit)
            .all(&state.sea_db)
            .await;

        match posts {
            Ok(list) => Ok(list),
            Err(err) => Err(err.into()),
        }
    }

    #[debug_handler]
    pub async fn rss(
        State(state): State<AppState>,
        Query(params): Query<FeedQuery>,
    ) -> Result<impl IntoResponse, ErrorResponse> {
        let site_url =
            std::env::var("SITE_URL").unwrap_or_else(|_| "http://localhost:8888".to_string());
        let site_name = std::env::var("SITE_NAME").unwrap_or_else(|_| "Ruxlog".to_string());

        let limit = params.limit.unwrap_or(20).min(100);
        let posts = fetch_latest_posts(&state, limit).await?;

        let updated = Utc::now().fixed_offset();
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str("<rss version=\"2.0\"><channel>");
        xml.push_str(&format!("<title>{}</title>", xml_escape(&site_name)));
        xml.push_str(&format!(
            "<link>{}</link>",
            xml_escape(&format!("{}/", site_url.trim_end_matches('/')))
        ));
        xml.push_str(&format!(
            "<description>{}</description>",
            xml_escape(&format!("Latest posts from {}", site_name))
        ));
        xml.push_str(&format!(
            "<lastBuildDate>{}</lastBuildDate>",
            updated.to_rfc2822()
        ));
        xml.push_str("<generator>ruxlog</generator>");

        for p in posts {
            let item_url = format!("{}/posts/{}", site_url.trim_end_matches('/'), p.slug);
            let pub_date = p.published_at.unwrap_or(p.updated_at).to_rfc2822();
            let title = xml_escape(&p.title);
            let desc_raw = if let Some(ex) = &p.excerpt {
                ex.clone()
            } else {
                content_to_summary(&p.content, 500)
            };
            let desc = xml_escape(&desc_raw);

            xml.push_str("<item>");
            xml.push_str(&format!("<title>{}</title>", title));
            xml.push_str(&format!("<link>{}</link>", xml_escape(&item_url)));
            xml.push_str(&format!(
                "<guid isPermaLink=\"true\">{}</guid>",
                xml_escape(&item_url)
            ));
            xml.push_str(&format!("<pubDate>{}</pubDate>", pub_date));
            xml.push_str(&format!("<description>{}</description>", desc));
            xml.push_str("</item>");
        }

        xml.push_str("</channel></rss>");

        build_xml_response("application/rss+xml; charset=utf-8", xml)
    }

    #[debug_handler]
    pub async fn atom(
        State(state): State<AppState>,
        Query(params): Query<FeedQuery>,
    ) -> Result<impl IntoResponse, ErrorResponse> {
        let site_url =
            std::env::var("SITE_URL").unwrap_or_else(|_| "http://localhost:8888".to_string());
        let site_name = std::env::var("SITE_NAME").unwrap_or_else(|_| "Ruxlog".to_string());

        let limit = params.limit.unwrap_or(20).min(100);
        let posts = fetch_latest_posts(&state, limit).await?;

        let updated = Utc::now().fixed_offset();
        let self_link = format!("{}/feed/v1/atom", site_url.trim_end_matches('/'));
        let home_link = format!("{}/", site_url.trim_end_matches('/'));
        let feed_id = format!("tag:{},{}", site_url, "feed:atom");

        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<feed xmlns="http://www.w3.org/2005/Atom">"#);
        xml.push_str(&format!("<title>{}</title>", xml_escape(&site_name)));
        xml.push_str(&format!("<id>{}</id>", xml_escape(&feed_id)));
        xml.push_str(&format!("<updated>{}</updated>", updated.to_rfc3339()));
        xml.push_str(&format!(
            r#"<link rel="self" href="{}" />"#,
            xml_escape(&self_link)
        ));
        xml.push_str(&format!(r#"<link href="{}" />"#, xml_escape(&home_link)));

        for p in posts {
            let entry_url = format!("{}/posts/{}", site_url.trim_end_matches('/'), p.slug);
            let pub_date = p.published_at.unwrap_or(p.updated_at).to_rfc3339();
            let title = xml_escape(&p.title);
            let summary_raw = if let Some(ex) = &p.excerpt {
                ex.clone()
            } else {
                content_to_summary(&p.content, 500)
            };
            let summary = xml_escape(&summary_raw);
            let entry_id = entry_url.clone();

            xml.push_str("<entry>");
            xml.push_str(&format!("<title>{}</title>", title));
            xml.push_str(&format!("<id>{}</id>", xml_escape(&entry_id)));
            xml.push_str(&format!(
                r#"<link rel="alternate" href="{}" />"#,
                xml_escape(&entry_url)
            ));
            xml.push_str(&format!("<updated>{}</updated>", pub_date));
            xml.push_str(&format!("<summary>{}</summary>", summary));
            xml.push_str("</entry>");
        }

        xml.push_str("</feed>");

        build_xml_response("application/atom+xml; charset=utf-8", xml)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // ── xml_escape ─────────────────────────────────────────────────

        #[test]
        fn xml_escape_ampersand() {
            assert_eq!(xml_escape("a&b"), "a&amp;b");
        }

        #[test]
        fn xml_escape_angle_brackets() {
            assert_eq!(xml_escape("<div>"), "&lt;div&gt;");
        }

        #[test]
        fn xml_escape_quotes() {
            assert_eq!(
                xml_escape(r#"say "hi" and it's fine"#),
                "say &quot;hi&quot; and it&apos;s fine"
            );
        }

        #[test]
        fn xml_escape_all_entities() {
            let input = "<tag attr='val'&more>";
            let escaped = xml_escape(input);
            assert!(escaped.contains("&lt;"));
            assert!(escaped.contains("&gt;"));
            assert!(escaped.contains("&apos;"));
            assert!(escaped.contains("&amp;"));
        }

        #[test]
        fn xml_escape_plain_string_unchanged() {
            assert_eq!(xml_escape("hello world"), "hello world");
        }

        #[test]
        fn xml_escape_empty() {
            assert_eq!(xml_escape(""), "");
        }

        // ── content_to_summary ─────────────────────────────────────────

        #[test]
        fn content_to_summary_paragraph() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "paragraph", "data": { "text": "Hello world" } }
                ]
            });
            assert_eq!(content_to_summary(&json, 100), "Hello world");
        }

        #[test]
        fn content_to_summary_header() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "header", "data": { "text": "Title" } }
                ]
            });
            assert_eq!(content_to_summary(&json, 100), "Title");
        }

        #[test]
        fn content_to_summary_quote() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "quote", "data": { "text": "To be or not to be" } }
                ]
            });
            assert_eq!(content_to_summary(&json, 100), "To be or not to be");
        }

        #[test]
        fn content_to_summary_alert() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "alert", "data": { "message": "Warning!" } }
                ]
            });
            assert_eq!(content_to_summary(&json, 100), "Warning!");
        }

        #[test]
        fn content_to_summary_checklist() {
            let json = serde_json::json!({
                "blocks": [
                    {
                        "type": "checklist",
                        "data": {
                            "items": [
                                { "text": "Buy milk" },
                                { "text": "Walk dog" }
                            ]
                        }
                    }
                ]
            });
            assert_eq!(content_to_summary(&json, 100), "Buy milk, Walk dog");
        }

        #[test]
        fn content_to_summary_code() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "code", "data": { "code": "fn main() {}" } }
                ]
            });
            assert_eq!(content_to_summary(&json, 100), "fn main() {}");
        }

        #[test]
        fn content_to_summary_unknown_type_skipped() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "image", "data": { "url": "http://example.com/img.png" } },
                    { "type": "paragraph", "data": { "text": "Visible" } }
                ]
            });
            let result = content_to_summary(&json, 100);
            assert_eq!(result, "Visible");
        }

        #[test]
        fn content_to_summary_multiple_blocks_joined() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "paragraph", "data": { "text": "First" } },
                    { "type": "paragraph", "data": { "text": "Second" } }
                ]
            });
            assert_eq!(content_to_summary(&json, 100), "First Second");
        }

        #[test]
        fn content_to_summary_respects_max_len() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "paragraph", "data": { "text": "A very long text that exceeds the limit" } }
                ]
            });
            let result = content_to_summary(&json, 10);
            // The function breaks once out.len() >= max_len (byte-based), then
            // truncates by chars().take(max_len). "A very lon" is 10 bytes and
            // 10 chars, so the 'g' never gets appended.
            assert_eq!(result, "A very lon");
        }

        #[test]
        fn content_to_summary_empty_blocks_falls_back_to_json_string() {
            let json = serde_json::json!({ "not": "blocks" });
            let result = content_to_summary(&json, 200);
            // Falls back to the entire JSON as a string
            assert!(result.contains("not"));
            assert!(result.contains("blocks"));
        }

        #[test]
        fn content_to_summary_empty_value() {
            let json = serde_json::json!({});
            let result = content_to_summary(&json, 100);
            // Fallback: the JSON representation of {}
            assert!(!result.is_empty());
        }

        #[test]
        fn content_to_summary_max_len_zero() {
            let json = serde_json::json!({
                "blocks": [
                    { "type": "paragraph", "data": { "text": "Hello" } }
                ]
            });
            assert_eq!(content_to_summary(&json, 0), "");
        }

        #[test]
        fn content_to_summary_truncates_fallback_json() {
            let json = serde_json::json!({
                "some_key": "some relatively long value here"
            });
            let result = content_to_summary(&json, 5);
            // The fallback JSON string is also truncated
            assert_eq!(result.len(), 5);
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/rss", get(controller::rss))
        .route("/atom", get(controller::atom))
}
