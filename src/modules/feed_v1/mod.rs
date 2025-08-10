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
            let desc_src = p.excerpt.as_deref().unwrap_or_else(|| p.content.as_str());
            let desc = xml_escape(&desc_src.chars().take(500).collect::<String>());

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
            let summary_src = p.excerpt.as_deref().unwrap_or_else(|| p.content.as_str());
            let summary = xml_escape(&summary_src.chars().take(500).collect::<String>());
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
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/rss", get(controller::rss))
        .route("/atom", get(controller::atom))
}
