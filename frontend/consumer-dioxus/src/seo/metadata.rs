use chrono::{DateTime, Utc};

/// Comprehensive SEO metadata for a page
#[derive(Clone, Debug, PartialEq)]
pub struct SeoMetadata {
    pub title: String,
    pub description: Option<String>,
    pub canonical_url: Option<String>,
    pub image: Option<SeoImage>,
    pub article: Option<ArticleMetadata>,
    pub robots: RobotsDirective,
    pub locale: String,
    pub site_name: String,
}

impl SeoMetadata {
    /// Get the Open Graph type based on whether this is an article
    pub fn og_type(&self) -> &'static str {
        if self.article.is_some() {
            "article"
        } else {
            "website"
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SeoImage {
    pub url: String,
    pub alt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArticleMetadata {
    pub published_time: DateTime<Utc>,
    pub modified_time: DateTime<Utc>,
    pub author: String,
    pub section: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum RobotsDirective {
    IndexFollow,
    NoIndexFollow,
    IndexNoFollow,
    NoIndexNoFollow,
}

impl RobotsDirective {
    pub fn to_string(&self) -> &'static str {
        match self {
            RobotsDirective::IndexFollow => "index, follow",
            RobotsDirective::NoIndexFollow => "noindex, follow",
            RobotsDirective::IndexNoFollow => "index, nofollow",
            RobotsDirective::NoIndexNoFollow => "noindex, nofollow",
        }
    }
}

/// Builder pattern for easy construction of SEO metadata
pub struct SeoMetadataBuilder {
    title: String,
    description: Option<String>,
    canonical_url: Option<String>,
    image: Option<SeoImage>,
    article: Option<ArticleMetadata>,
    robots: RobotsDirective,
    locale: String,
    site_name: String,
}

impl SeoMetadataBuilder {
    pub fn new() -> Self {
        use crate::seo::config::SEO_CONFIG;

        Self {
            title: String::new(),
            description: None,
            canonical_url: None,
            image: None,
            article: None,
            robots: RobotsDirective::IndexFollow,
            locale: SEO_CONFIG.locale.to_string(),
            site_name: SEO_CONFIG.site_name.to_string(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn canonical(mut self, path: &str) -> Self {
        use crate::seo::config;
        self.canonical_url = Some(config::canonical_url(path));
        self
    }

    pub fn canonical_url(mut self, url: String) -> Self {
        self.canonical_url = Some(url);
        self
    }

    pub fn image(mut self, url: &str, alt: &str) -> Self {
        self.image = Some(SeoImage {
            url: url.to_string(),
            alt: alt.to_string(),
            width: None,
            height: None,
        });
        self
    }

    pub fn image_with_dimensions(mut self, url: &str, alt: &str, width: u32, height: u32) -> Self {
        self.image = Some(SeoImage {
            url: url.to_string(),
            alt: alt.to_string(),
            width: Some(width),
            height: Some(height),
        });
        self
    }

    pub fn image_struct(mut self, image: Option<SeoImage>) -> Self {
        self.image = image;
        self
    }

    pub fn article(mut self, article: ArticleMetadata) -> Self {
        self.article = Some(article);
        self
    }

    pub fn robots(mut self, robots: RobotsDirective) -> Self {
        self.robots = robots;
        self
    }

    pub fn locale(mut self, locale: &str) -> Self {
        self.locale = locale.to_string();
        self
    }

    pub fn build(self) -> SeoMetadata {
        SeoMetadata {
            title: self.title,
            description: self.description,
            canonical_url: self.canonical_url,
            image: self.image,
            article: self.article,
            robots: self.robots,
            locale: self.locale,
            site_name: self.site_name,
        }
    }
}

impl Default for SeoMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}
