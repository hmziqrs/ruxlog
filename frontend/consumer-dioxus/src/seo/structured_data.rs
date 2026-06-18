use dioxus::prelude::*;
use serde_json::{json, Value};

use super::config::SEO_CONFIG;
use ruxlog_shared::Post;

/// Serialize JSON-LD for safe embedding inside a
/// `<script type="application/ld+json">…</script>` element.
///
/// `serde_json::to_string` does not escape `<`, `>` or `&`, so any
/// attacker/admin-controlled string interpolated into the schema (post titles,
/// tag/category/author names, breadcrumb labels) could otherwise contain
/// `</script>` and terminate the script context — enabling markup injection in
/// the page (plan Phase 6e). Escaping the three HTML-significant characters as
/// JSON Unicode escapes keeps the JSON valid while making it inert inside an
/// HTML script element.
fn to_safe_json_ld(value: &Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "{}".to_string())
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

/// Generate Article schema JSON-LD for blog posts
pub fn article_schema(post: &Post) -> String {
    let schema = json!({
        "@context": "https://schema.org",
        "@type": "BlogPosting",
        "headline": post.title,
        "description": post.excerpt,
        "image": post.featured_image.as_ref().map(|img| &img.file_url),
        "datePublished": post.published_at.unwrap_or(post.created_at).to_rfc3339(),
        "dateModified": post.updated_at.to_rfc3339(),
        "author": {
            "@type": "Person",
            "name": &post.author.name,
            "url": SEO_CONFIG.consumer_url
        },
        "publisher": {
            "@type": "Organization",
            "name": SEO_CONFIG.site_name,
            "logo": {
                "@type": "ImageObject",
                "url": format!("{}/logo.png", SEO_CONFIG.consumer_url)
            }
        },
        "mainEntityOfPage": {
            "@type": "WebPage",
            "@id": format!("{}/posts/{}", SEO_CONFIG.consumer_url, post.slug)
        },
        "articleSection": &post.category.name,
        "keywords": post.tags.iter().map(|t| &t.name).collect::<Vec<_>>()
    });

    to_safe_json_ld(&schema)
}

/// Generate BreadcrumbList schema
pub fn breadcrumb_schema(items: Vec<(&str, &str)>) -> String {
    let list_items: Vec<_> = items
        .iter()
        .enumerate()
        .map(|(index, (name, url))| {
            json!({
                "@type": "ListItem",
                "position": index + 1,
                "name": name,
                "item": if url.starts_with("http") {
                    url.to_string()
                } else {
                    format!("{}{}", SEO_CONFIG.consumer_url, url)
                }
            })
        })
        .collect();

    let schema = json!({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": list_items
    });

    to_safe_json_ld(&schema)
}

/// Generate WebSite schema for homepage
pub fn website_schema() -> String {
    let schema = json!({
        "@context": "https://schema.org",
        "@type": "WebSite",
        "name": SEO_CONFIG.site_name,
        "description": SEO_CONFIG.site_tagline,
        "url": SEO_CONFIG.consumer_url,
        "publisher": {
            "@type": "Organization",
            "name": SEO_CONFIG.site_name,
            "logo": {
                "@type": "ImageObject",
                "url": format!("{}/logo.png", SEO_CONFIG.consumer_url)
            }
        }
    });

    to_safe_json_ld(&schema)
}

/// Component to inject JSON-LD structured data into the page
#[component]
pub fn StructuredData(json_ld: String) -> Element {
    rsx! {
        script {
            r#type: "application/ld+json",
            dangerous_inner_html: "{json_ld}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_breadcrumb_schema_valid_json() {
        let items = vec![("Home", "/"), ("Blog", "/blog"), ("Post", "/blog/post-1")];
        let schema = breadcrumb_schema(items);

        // Verify it's valid JSON
        let parsed: Result<Value, _> = serde_json::from_str(&schema);
        assert!(parsed.is_ok());

        let value = parsed.unwrap();
        assert_eq!(value["@type"], "BreadcrumbList");
        assert_eq!(value["itemListElement"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_website_schema_valid_json() {
        let schema = website_schema();

        // Verify it's valid JSON
        let parsed: Result<Value, _> = serde_json::from_str(&schema);
        assert!(parsed.is_ok());

        let value = parsed.unwrap();
        assert_eq!(value["@type"], "WebSite");
        assert_eq!(value["name"], SEO_CONFIG.site_name);
    }
}
