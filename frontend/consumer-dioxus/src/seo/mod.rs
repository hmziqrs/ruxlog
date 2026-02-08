pub mod components;
pub mod config;
pub mod hooks;
pub mod metadata;
pub mod structured_data;
pub mod utils;

// Re-export commonly used types and functions
pub use components::SeoHead;
pub use config::{canonical_url, format_title, truncate_description, SEO_CONFIG};
pub use hooks::{
    use_category_seo, use_post_seo, use_post_seo_by_slug, use_static_seo, use_tag_seo,
};
pub use metadata::{ArticleMetadata, RobotsDirective, SeoImage, SeoMetadata, SeoMetadataBuilder};
pub use structured_data::{article_schema, breadcrumb_schema, website_schema, StructuredData};
pub use utils::{clean_text, ensure_absolute_url, extract_first_paragraph, generate_excerpt};
