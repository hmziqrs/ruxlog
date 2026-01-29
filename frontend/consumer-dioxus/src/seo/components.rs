use dioxus::prelude::*;

use super::config::{format_title, SEO_CONFIG};
use super::metadata::SeoMetadata;

#[component]
pub fn SeoHead(metadata: SeoMetadata) -> Element {
    let title = format_title(&metadata.title);
    let description = metadata
        .description
        .clone()
        .unwrap_or_else(|| SEO_CONFIG.default_description.to_string());

    let image_url = metadata
        .image
        .as_ref()
        .map(|img| {
            if img.url.starts_with("http") {
                img.url.clone()
            } else {
                format!("{}{}", SEO_CONFIG.consumer_url, img.url)
            }
        })
        .unwrap_or_else(|| format!("{}{}", SEO_CONFIG.consumer_url, SEO_CONFIG.default_image));

    let canonical = metadata.canonical_url.clone().unwrap_or_else(|| {
        // Fallback: try to get current URL from window.location
        SEO_CONFIG.consumer_url.to_string()
    });

    let robots_content = metadata.robots.to_string();

    rsx! {
        // Basic meta tags
        document::Title { "{title}" }
        document::Meta { name: "description", content: description.clone() }
        document::Meta { name: "robots", content: robots_content }

        // Canonical URL
        document::Link { rel: "canonical", href: canonical.clone() }

        // Open Graph tags
        document::Meta { property: "og:type", content: metadata.og_type() }
        document::Meta { property: "og:title", content: title.clone() }
        document::Meta { property: "og:description", content: description.clone() }
        document::Meta { property: "og:image", content: image_url.clone() }
        document::Meta { property: "og:url", content: canonical.clone() }
        document::Meta { property: "og:site_name", content: SEO_CONFIG.site_name }
        document::Meta { property: "og:locale", content: metadata.locale.clone() }

        // Article-specific OG tags
        if let Some(article) = &metadata.article {
            document::Meta {
                property: "article:published_time",
                content: article.published_time.to_rfc3339()
            }
            document::Meta {
                property: "article:modified_time",
                content: article.modified_time.to_rfc3339()
            }
            document::Meta { property: "article:author", content: article.author.clone() }

            if let Some(section) = &article.section {
                document::Meta { property: "article:section", content: section.clone() }
            }

            for tag in &article.tags {
                document::Meta { property: "article:tag", content: tag.clone() }
            }
        }

        // Image dimensions if available
        if let Some(image) = &metadata.image {
            if let Some(width) = image.width {
                document::Meta { property: "og:image:width", content: width.to_string() }
            }
            if let Some(height) = image.height {
                document::Meta { property: "og:image:height", content: height.to_string() }
            }
            document::Meta { property: "og:image:alt", content: image.alt.clone() }
        }

        // Twitter Card tags
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:title", content: title }
        document::Meta { name: "twitter:description", content: description }
        document::Meta { name: "twitter:image", content: image_url }

        if let Some(handle) = SEO_CONFIG.twitter_handle {
            document::Meta { name: "twitter:site", content: handle }
            document::Meta { name: "twitter:creator", content: handle }
        }
    }
}
