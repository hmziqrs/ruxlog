use dioxus::prelude::*;
use serde_json::json;

use super::config::SEO_CONFIG;
use ruxlog_shared::Post;

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

    serde_json::to_string(&schema).unwrap_or_else(|_| "{}".to_string())
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

    serde_json::to_string(&schema).unwrap_or_else(|_| "{}".to_string())
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

    serde_json::to_string(&schema).unwrap_or_else(|_| "{}".to_string())
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
