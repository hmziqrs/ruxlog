use dioxus::prelude::*;

use super::config::{truncate_description, SEO_CONFIG};
use super::metadata::{ArticleMetadata, SeoImage, SeoMetadata, SeoMetadataBuilder};

use ruxlog_shared::store::{use_categories, use_post, use_tag};

/// Auto-generate SEO metadata from a post by ID
/// Returns None if post is not found or not yet loaded
pub fn use_post_seo(post_id: i32) -> Memo<Option<SeoMetadata>> {
    let posts = use_post();

    use_memo(move || {
        if let Some(frame) = posts.view.read().get(&post_id) {
            if let Some(post) = frame.data.as_ref() {
                return Some(build_post_seo(post));
            }
        }

        let posts_read = posts.list.read();
        if let Some(list) = &(*posts_read).data {
            if let Some(post) = list.data.iter().find(|p| p.id == post_id) {
                return Some(build_post_seo(post));
            }
        }
        None
    })
}

/// Auto-generate SEO metadata from a post by slug
/// Returns None if post is not found or not yet loaded
pub fn use_post_seo_by_slug(slug: String) -> Memo<Option<SeoMetadata>> {
    let posts = use_post();

    use_memo(move || {
        let view_map = posts.view.read();
        if let Some(post) = view_map.values().find_map(|frame| {
            frame
                .data
                .as_ref()
                .filter(|p| p.slug.as_str() == slug.as_str())
        }) {
            return Some(build_post_seo(post));
        }

        let posts_read = posts.list.read();
        if let Some(list) = &(*posts_read).data {
            if let Some(post) = list.data.iter().find(|p| p.slug.as_str() == slug.as_str()) {
                return Some(build_post_seo(post));
            }
        }

        None
    })
}

/// Build SEO metadata from a Post instance
fn build_post_seo(post: &ruxlog_shared::Post) -> SeoMetadata {
    // Use excerpt if available, otherwise extract from content
    let description = post
        .excerpt
        .clone()
        .or_else(|| {
            // Try to extract first paragraph from EditorJS content
            let content_text = extract_text_from_editorjs(&post.content);
            content_text.and_then(|text| if text.len() > 20 { Some(text) } else { None })
        })
        .unwrap_or_else(|| format!("Read {} on {}", post.title, SEO_CONFIG.site_name));

    // Truncate description to SEO-optimal length
    let description = truncate_description(&description, 160);

    // Build SEO image from featured image
    let image = post.featured_image.as_ref().map(|img| SeoImage {
        url: img.file_url.clone(),
        alt: post.title.clone(),
        width: img.width.map(|w| w as u32),
        height: img.height.map(|h| h as u32),
    });

    // Build article metadata
    let article = ArticleMetadata {
        published_time: post.published_at.unwrap_or(post.created_at),
        modified_time: post.updated_at,
        author: post.author.name.clone(),
        section: Some(post.category.name.clone()),
        tags: post.tags.iter().map(|t| t.name.clone()).collect(),
    };

    SeoMetadataBuilder::new()
        .title(&post.title)
        .description(&description)
        .image_struct(image)
        .article(article)
        .canonical(&format!("/posts/{}", post.slug))
        .build()
}

/// Extract text from EditorJS content for description
fn extract_text_from_editorjs(content: &ruxlog_shared::PostContent) -> Option<String> {
    use ruxlog_shared::EditorJsBlock;

    for block in &content.blocks {
        match block {
            EditorJsBlock::Paragraph { data, .. } => {
                if !data.text.is_empty() {
                    return Some(strip_html_tags(&data.text));
                }
            }
            EditorJsBlock::Header { data, .. } => {
                if !data.text.is_empty() {
                    return Some(strip_html_tags(&data.text));
                }
            }
            _ => continue,
        }
    }

    None
}

/// Simple HTML tag stripper
fn strip_html_tags(html: &str) -> String {
    use regex::Regex;
    let re = Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(html, "").trim().to_string()
}

/// Auto-generate SEO metadata from a category by slug
pub fn use_category_seo(slug: String) -> Memo<Option<SeoMetadata>> {
    let categories = use_categories();

    use_memo(move || {
        let categories_read = categories.list.read();
        if let Some(list) = &(*categories_read).data {
            if let Some(category) = list.data.iter().find(|c| c.slug == slug) {
                return Some(build_category_seo(category));
            }
        }
        None
    })
}

/// Build SEO metadata from a Category instance
fn build_category_seo(category: &ruxlog_shared::Category) -> SeoMetadata {
    let description = category.description.clone().unwrap_or_else(|| {
        format!(
            "Browse posts in the {} category on {}",
            category.name, SEO_CONFIG.site_name
        )
    });

    let description = truncate_description(&description, 160);

    // Use category cover or logo as image
    let image = category
        .cover
        .as_ref()
        .or(category.logo.as_ref())
        .map(|img| SeoImage {
            url: img.file_url.clone(),
            alt: format!("{} category", category.name),
            width: img.width.map(|w| w as u32),
            height: img.height.map(|h| h as u32),
        });

    SeoMetadataBuilder::new()
        .title(&format!("{} Category", category.name))
        .description(&description)
        .image_struct(image)
        .canonical(&format!("/categories/{}", category.slug))
        .build()
}

/// Auto-generate SEO metadata from a tag by slug
pub fn use_tag_seo(slug: String) -> Memo<Option<SeoMetadata>> {
    let tags = use_tag();

    use_memo(move || {
        let tags_read = tags.list.read();
        if let Some(list) = &(*tags_read).data {
            if let Some(tag) = list.data.iter().find(|t| t.slug == slug) {
                return Some(build_tag_seo(tag));
            }
        }
        None
    })
}

/// Build SEO metadata from a Tag instance
fn build_tag_seo(tag: &ruxlog_shared::Tag) -> SeoMetadata {
    let description = tag
        .description
        .clone()
        .unwrap_or_else(|| format!("Posts tagged with {} on {}", tag.name, SEO_CONFIG.site_name));

    let description = truncate_description(&description, 160);

    SeoMetadataBuilder::new()
        .title(&format!("{} Tag", tag.name))
        .description(&description)
        .canonical(&format!("/tags/{}", tag.slug))
        .build()
}

/// Predefined SEO metadata for static pages
pub fn use_static_seo(page: &str) -> SeoMetadata {
    match page {
        "about" => SeoMetadataBuilder::new()
            .title("About")
            .description(&format!(
                "Learn about {} and the developer behind it",
                SEO_CONFIG.site_name
            ))
            .canonical("/about")
            .build(),
        "contact" => SeoMetadataBuilder::new()
            .title("Contact")
            .description(&format!(
                "Get in touch with the {} team",
                SEO_CONFIG.site_name
            ))
            .canonical("/contact")
            .build(),
        "privacy" => SeoMetadataBuilder::new()
            .title("Privacy Policy")
            .description(&format!(
                "Privacy policy for {} - how we collect, use, and protect your data",
                SEO_CONFIG.site_name
            ))
            .canonical("/privacy")
            .build(),
        "terms" => SeoMetadataBuilder::new()
            .title("Terms of Service")
            .description(&format!(
                "Terms of service for {} - rules and guidelines for using our platform",
                SEO_CONFIG.site_name
            ))
            .canonical("/terms")
            .build(),
        "advertise" => SeoMetadataBuilder::new()
            .title("Advertise")
            .description(&format!(
                "Advertise on {} - reach our engaged audience of developers and tech enthusiasts",
                SEO_CONFIG.site_name
            ))
            .canonical("/advertise")
            .build(),
        "home" => SeoMetadataBuilder::new()
            .title("")
            .description(SEO_CONFIG.default_description)
            .canonical("/")
            .build(),
        _ => default_seo(),
    }
}

/// Default SEO metadata fallback
fn default_seo() -> SeoMetadata {
    SeoMetadataBuilder::new()
        .title("")
        .description(SEO_CONFIG.default_description)
        .canonical("/")
        .build()
}
