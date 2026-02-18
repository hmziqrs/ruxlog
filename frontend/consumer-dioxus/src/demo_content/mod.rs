use chrono::{DateTime, Utc};
use include_dir::{include_dir, Dir};
use oxstore::PaginatedList;
use pulldown_cmark::{html, Options, Parser};
use ruxlog_shared::store::{
    Category, EditorJsBlock, Media, Post, PostAuthor, PostCategory, PostContent, PostStatus,
    PostTag, RawBlock, Tag,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

const DEFAULT_CATEGORY_COLOR: &str = "#f97316";
const DEFAULT_TAG_COLOR: &str = "#3b82f6";
const DEFAULT_TEXT_COLOR: &str = "#ffffff";

static POSTS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/content/posts");
static DEMO_CONTENT: OnceLock<DemoContent> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct DemoContent {
    posts_by_slug: HashMap<String, Post>,
    posts_sorted: Vec<Post>,
    tags_by_slug: HashMap<String, Tag>,
    tags_sorted: Vec<Tag>,
    categories_by_slug: HashMap<String, Category>,
    categories_sorted: Vec<Category>,
    posts_by_tag_id: HashMap<i32, Vec<Post>>,
    posts_by_category_id: HashMap<i32, Vec<Post>>,
}

impl DemoContent {
    fn load() -> Result<Self, String> {
        let sources = embedded_markdown_sources()?;
        parse_sources(sources)
    }

    pub fn posts(&self) -> &[Post] {
        &self.posts_sorted
    }

    pub fn post_by_slug(&self, slug: &str) -> Option<Post> {
        self.posts_by_slug.get(slug).cloned()
    }

    pub fn tags(&self) -> &[Tag] {
        &self.tags_sorted
    }

    pub fn tag_by_slug(&self, slug: &str) -> Option<Tag> {
        self.tags_by_slug.get(slug).cloned()
    }

    pub fn categories(&self) -> &[Category] {
        &self.categories_sorted
    }

    pub fn category_by_slug(&self, slug: &str) -> Option<Category> {
        self.categories_by_slug.get(slug).cloned()
    }

    pub fn posts_by_tag_id(&self, tag_id: i32) -> Vec<Post> {
        self.posts_by_tag_id
            .get(&tag_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn posts_by_category_id(&self, category_id: i32) -> Vec<Post> {
        self.posts_by_category_id
            .get(&category_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn dynamic_routes(&self) -> Vec<String> {
        let mut routes = Vec::with_capacity(
            self.posts_by_slug.len() + self.tags_by_slug.len() + self.categories_by_slug.len(),
        );

        routes.extend(
            self.posts_by_slug
                .keys()
                .map(|slug| format!("/posts/{slug}")),
        );
        routes.extend(self.tags_by_slug.keys().map(|slug| format!("/tags/{slug}")));
        routes.extend(
            self.categories_by_slug
                .keys()
                .map(|slug| format!("/categories/{slug}")),
        );

        routes.sort();
        routes
    }
}

pub fn content() -> &'static DemoContent {
    DEMO_CONTENT.get_or_init(|| {
        DemoContent::load()
            .unwrap_or_else(|err| panic!("Failed to load demo markdown content: {err}"))
    })
}

pub fn paginated<T: Clone>(items: &[T]) -> PaginatedList<T> {
    PaginatedList {
        data: items.to_vec(),
        total: items.len() as u64,
        page: 1,
        per_page: std::cmp::max(items.len(), 1) as u64,
    }
}

#[derive(Debug, Clone)]
struct SourceFile {
    path: String,
    contents: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Frontmatter {
    title: String,
    slug: String,
    excerpt: Option<String>,
    published_at: DateTime<Utc>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
    author: FrontmatterAuthor,
    category: FrontmatterCategory,
    #[serde(default)]
    tags: Vec<FrontmatterTag>,
    featured_image: FrontmatterFeaturedImage,
}

#[derive(Debug, Clone, Deserialize)]
struct FrontmatterAuthor {
    name: String,
    email: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FrontmatterCategory {
    name: String,
    slug: String,
    #[serde(default)]
    color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FrontmatterTag {
    name: String,
    slug: String,
    #[serde(default)]
    color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FrontmatterFeaturedImage {
    file_url: String,
    #[serde(default)]
    width: Option<i32>,
    #[serde(default)]
    height: Option<i32>,
}

#[derive(Debug, Clone)]
struct ParsedSource {
    path: String,
    frontmatter: Frontmatter,
    body_markdown: String,
}

fn embedded_markdown_sources() -> Result<Vec<SourceFile>, String> {
    let mut files = Vec::new();
    collect_markdown_files(&POSTS_DIR, &mut files);

    if files.is_empty() {
        return Err("No markdown files found under content/posts".to_string());
    }

    let mut sources = Vec::with_capacity(files.len());
    for file in files {
        let path = file.path().display().to_string();
        let contents = file
            .contents_utf8()
            .ok_or_else(|| format!("File is not valid UTF-8: {path}"))?
            .to_string();

        sources.push(SourceFile { path, contents });
    }

    Ok(sources)
}

fn collect_markdown_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a include_dir::File<'a>>) {
    for file in dir.files() {
        if file.path().extension().and_then(|ext| ext.to_str()) == Some("md") {
            out.push(file);
        }
    }

    for child in dir.dirs() {
        collect_markdown_files(child, out);
    }
}

fn parse_sources(mut sources: Vec<SourceFile>) -> Result<DemoContent, String> {
    sources.sort_by(|a, b| a.path.cmp(&b.path));

    let mut parsed = Vec::with_capacity(sources.len());
    for source in sources {
        parsed.push(parse_source_file(&source)?);
    }

    if parsed.is_empty() {
        return Err("At least one markdown post is required".to_string());
    }

    parsed.sort_by(|a, b| a.frontmatter.slug.cmp(&b.frontmatter.slug));

    ensure_unique_post_slugs(&parsed)?;

    let author_ids = build_author_ids(&parsed)?;
    let categories = build_categories(&parsed)?;
    let tags = build_tags(&parsed)?;

    let category_index: HashMap<String, Category> = categories
        .iter()
        .map(|item| (item.slug.clone(), item.clone()))
        .collect();
    let tag_index: HashMap<String, Tag> = tags
        .iter()
        .map(|item| (item.slug.clone(), item.clone()))
        .collect();

    let mut posts = Vec::with_capacity(parsed.len());
    for (offset, source) in parsed.iter().enumerate() {
        let post_id = (offset + 1) as i32;
        posts.push(build_post(
            post_id,
            source,
            &author_ids,
            &category_index,
            &tag_index,
        )?);
    }

    sort_posts_for_display(&mut posts);

    let posts_by_slug = posts
        .iter()
        .map(|post| (post.slug.clone(), post.clone()))
        .collect::<HashMap<_, _>>();

    let mut posts_by_tag_id: HashMap<i32, Vec<Post>> = HashMap::new();
    let mut posts_by_category_id: HashMap<i32, Vec<Post>> = HashMap::new();

    for post in &posts {
        posts_by_category_id
            .entry(post.category.id)
            .or_default()
            .push(post.clone());

        for tag in &post.tags {
            posts_by_tag_id
                .entry(tag.id)
                .or_default()
                .push(post.clone());
        }
    }

    for values in posts_by_category_id.values_mut() {
        sort_posts_for_display(values);
    }

    for values in posts_by_tag_id.values_mut() {
        sort_posts_for_display(values);
    }

    let categories_by_slug = categories
        .iter()
        .map(|item| (item.slug.clone(), item.clone()))
        .collect::<HashMap<_, _>>();

    let tags_by_slug = tags
        .iter()
        .map(|item| (item.slug.clone(), item.clone()))
        .collect::<HashMap<_, _>>();

    Ok(DemoContent {
        posts_by_slug,
        posts_sorted: posts,
        tags_by_slug,
        tags_sorted: tags,
        categories_by_slug,
        categories_sorted: categories,
        posts_by_tag_id,
        posts_by_category_id,
    })
}

fn parse_source_file(source: &SourceFile) -> Result<ParsedSource, String> {
    let (frontmatter_raw, body_markdown) =
        split_frontmatter(&source.contents).map_err(|err| format!("{}: {err}", source.path))?;

    let frontmatter =
        parse_frontmatter(frontmatter_raw).map_err(|err| format!("{}: {err}", source.path))?;

    validate_frontmatter(&frontmatter).map_err(|err| format!("{}: {err}", source.path))?;

    Ok(ParsedSource {
        path: source.path.clone(),
        frontmatter,
        body_markdown: body_markdown.to_string(),
    })
}

fn split_frontmatter(contents: &str) -> Result<(&str, &str), String> {
    let rest = contents
        .strip_prefix("---\n")
        .ok_or_else(|| "Missing opening frontmatter delimiter `---`".to_string())?;

    let delimiter = "\n---\n";
    let Some(end) = rest.find(delimiter) else {
        return Err("Missing closing frontmatter delimiter `---`".to_string());
    };

    let frontmatter = &rest[..end];
    let body = &rest[end + delimiter.len()..];

    if body.trim().is_empty() {
        return Err("Markdown body cannot be empty".to_string());
    }

    Ok((frontmatter, body))
}

fn parse_frontmatter(frontmatter_raw: &str) -> Result<Frontmatter, String> {
    serde_yaml::from_str(frontmatter_raw).map_err(|err| format!("Invalid frontmatter YAML: {err}"))
}

fn validate_frontmatter(frontmatter: &Frontmatter) -> Result<(), String> {
    if frontmatter.title.trim().is_empty() {
        return Err("`title` is required".to_string());
    }

    if frontmatter.slug.trim().is_empty() {
        return Err("`slug` is required".to_string());
    }

    if frontmatter.author.name.trim().is_empty() {
        return Err("`author.name` is required".to_string());
    }

    if frontmatter.author.email.trim().is_empty() {
        return Err("`author.email` is required".to_string());
    }

    if frontmatter.category.name.trim().is_empty() {
        return Err("`category.name` is required".to_string());
    }

    if frontmatter.category.slug.trim().is_empty() {
        return Err("`category.slug` is required".to_string());
    }

    if frontmatter.featured_image.file_url.trim().is_empty() {
        return Err("`featured_image.file_url` is required".to_string());
    }

    for (index, tag) in frontmatter.tags.iter().enumerate() {
        if tag.name.trim().is_empty() {
            return Err(format!("`tags[{index}].name` is required"));
        }

        if tag.slug.trim().is_empty() {
            return Err(format!("`tags[{index}].slug` is required"));
        }
    }

    Ok(())
}

fn ensure_unique_post_slugs(sources: &[ParsedSource]) -> Result<(), String> {
    let mut seen = BTreeSet::new();

    for source in sources {
        if !seen.insert(source.frontmatter.slug.clone()) {
            return Err(format!(
                "Duplicate post slug '{}' in {}",
                source.frontmatter.slug, source.path
            ));
        }
    }

    Ok(())
}

fn build_author_ids(sources: &[ParsedSource]) -> Result<HashMap<String, i32>, String> {
    let mut authors = BTreeMap::new();

    for source in sources {
        let email = source.frontmatter.author.email.trim().to_lowercase();
        let name = source.frontmatter.author.name.trim().to_string();

        match authors.get(&email) {
            Some(existing_name) if existing_name != &name => {
                return Err(format!(
                    "Author email '{}' has multiple names ('{}' and '{}')",
                    email, existing_name, name
                ));
            }
            _ => {
                authors.insert(email, name);
            }
        }
    }

    let mut ids = HashMap::new();
    for (index, (email, _name)) in authors.into_iter().enumerate() {
        ids.insert(email, (index + 1) as i32);
    }

    Ok(ids)
}

fn build_categories(sources: &[ParsedSource]) -> Result<Vec<Category>, String> {
    let mut categories: BTreeMap<String, (String, String)> = BTreeMap::new();

    for source in sources {
        let slug = source.frontmatter.category.slug.trim().to_string();
        let name = source.frontmatter.category.name.trim().to_string();
        let color = source
            .frontmatter
            .category
            .color
            .clone()
            .unwrap_or_else(|| DEFAULT_CATEGORY_COLOR.to_string());

        match categories.get(&slug) {
            Some((existing_name, existing_color))
                if existing_name != &name || existing_color != &color =>
            {
                return Err(format!(
                    "Category '{}' has conflicting metadata across posts",
                    slug
                ));
            }
            _ => {
                categories.insert(slug, (name, color));
            }
        }
    }

    let now = Utc::now();
    let mut output = Vec::with_capacity(categories.len());

    for (index, (slug, (name, color))) in categories.into_iter().enumerate() {
        output.push(Category {
            id: (index + 1) as i32,
            name,
            slug,
            created_at: now,
            updated_at: now,
            color,
            text_color: DEFAULT_TEXT_COLOR.to_string(),
            is_active: true,
            cover: None,
            logo: None,
            cover_id: None,
            description: None,
            logo_id: None,
            parent_id: None,
        });
    }

    Ok(output)
}

fn build_tags(sources: &[ParsedSource]) -> Result<Vec<Tag>, String> {
    let mut tags: BTreeMap<String, (String, String)> = BTreeMap::new();

    for source in sources {
        for tag in &source.frontmatter.tags {
            let slug = tag.slug.trim().to_string();
            let name = tag.name.trim().to_string();
            let color = tag
                .color
                .clone()
                .unwrap_or_else(|| DEFAULT_TAG_COLOR.to_string());

            match tags.get(&slug) {
                Some((existing_name, existing_color))
                    if existing_name != &name || existing_color != &color =>
                {
                    return Err(format!(
                        "Tag '{}' has conflicting metadata across posts",
                        slug
                    ));
                }
                _ => {
                    tags.insert(slug, (name, color));
                }
            }
        }
    }

    let now = Utc::now();
    let mut output = Vec::with_capacity(tags.len());

    for (index, (slug, (name, color))) in tags.into_iter().enumerate() {
        output.push(Tag {
            id: (index + 1) as i32,
            name,
            slug,
            created_at: now,
            updated_at: now,
            description: None,
            color,
            text_color: DEFAULT_TEXT_COLOR.to_string(),
            is_active: true,
        });
    }

    Ok(output)
}

fn build_post(
    post_id: i32,
    source: &ParsedSource,
    author_ids: &HashMap<String, i32>,
    categories_by_slug: &HashMap<String, Category>,
    tags_by_slug: &HashMap<String, Tag>,
) -> Result<Post, String> {
    let frontmatter = &source.frontmatter;

    let author_email = frontmatter.author.email.trim().to_lowercase();
    let author_id = author_ids.get(&author_email).ok_or_else(|| {
        format!(
            "Missing generated author id for email '{}' in {}",
            author_email, source.path
        )
    })?;

    let category = categories_by_slug
        .get(frontmatter.category.slug.trim())
        .ok_or_else(|| {
            format!(
                "Unknown category '{}' referenced by {}",
                frontmatter.category.slug, source.path
            )
        })?;

    let mut tags = Vec::with_capacity(frontmatter.tags.len());
    for tag in &frontmatter.tags {
        let full_tag = tags_by_slug
            .get(tag.slug.trim())
            .ok_or_else(|| format!("Unknown tag '{}' referenced by {}", tag.slug, source.path))?;

        tags.push(PostTag {
            id: full_tag.id,
            name: full_tag.name.clone(),
            slug: full_tag.slug.clone(),
            color: full_tag.color.clone(),
        });
    }

    let published_at = frontmatter.published_at;
    let updated_at = frontmatter.updated_at.unwrap_or(frontmatter.published_at);

    let post_content = PostContent {
        time: (published_at.timestamp_millis().max(0)) as u64,
        blocks: vec![EditorJsBlock::Raw {
            id: Some("markdown-body".to_string()),
            data: RawBlock {
                html: markdown_to_sanitized_html(&source.body_markdown),
            },
        }],
        version: "markdown-demo-v1".to_string(),
    };

    Ok(Post {
        id: post_id,
        title: frontmatter.title.clone(),
        content: post_content,
        slug: frontmatter.slug.clone(),
        excerpt: frontmatter.excerpt.clone(),
        featured_image: Some(build_featured_media(post_id, &frontmatter.featured_image)),
        published_at: Some(published_at),
        created_at: published_at,
        updated_at,
        author: PostAuthor {
            id: *author_id,
            name: frontmatter.author.name.clone(),
            email: frontmatter.author.email.clone(),
            avatar: None,
        },
        category: PostCategory {
            id: category.id,
            name: category.name.clone(),
            slug: category.slug.clone(),
            color: category.color.clone(),
            logo: None,
            cover: None,
        },
        tags,
        likes_count: 0,
        view_count: 0,
        comment_count: 0,
        status: PostStatus::Published,
    })
}

fn markdown_to_sanitized_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    ammonia::Builder::default().clean(&html_output).to_string()
}

fn build_featured_media(post_id: i32, featured_image: &FrontmatterFeaturedImage) -> Media {
    let file_url = featured_image.file_url.trim().to_string();
    let extension = file_url
        .rsplit_once('.')
        .map(|(_prefix, ext)| ext.to_ascii_lowercase());

    let mime_type = match extension.as_deref() {
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("avif") => "image/avif",
        Some("svg") => "image/svg+xml",
        _ => "image/jpeg",
    }
    .to_string();

    let now = Utc::now();

    Media {
        id: post_id,
        object_key: file_url.trim_start_matches('/').to_string(),
        file_url,
        mime_type,
        size: 0,
        width: featured_image.width,
        height: featured_image.height,
        extension,
        uploader_id: None,
        reference_type: None,
        content_hash: None,
        is_optimized: false,
        optimized_at: None,
        usage_count: 0,
        created_at: now,
        updated_at: now,
    }
}

fn sort_posts_for_display(posts: &mut [Post]) {
    posts.sort_by(|left, right| {
        right
            .published_at
            .unwrap_or(right.created_at)
            .cmp(&left.published_at.unwrap_or(left.created_at))
            .then_with(|| left.slug.cmp(&right.slug))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_source(path: &str, markdown: &str) -> SourceFile {
        SourceFile {
            path: path.to_string(),
            contents: markdown.to_string(),
        }
    }

    fn valid_markdown(slug: &str) -> String {
        format!(
            "---\ntitle: \"Post {slug}\"\nslug: \"{slug}\"\nexcerpt: \"Excerpt for {slug}\"\npublished_at: \"2026-02-18T12:00:00Z\"\nauthor:\n  name: \"Demo Author\"\n  email: \"demo@example.com\"\ncategory:\n  name: \"Rust\"\n  slug: \"rust\"\ntags:\n  - name: \"Dioxus\"\n    slug: \"dioxus\"\nfeatured_image:\n  file_url: \"/assets/logo.png\"\n  width: 512\n  height: 512\n---\n# Heading\n\nBody text.\n"
        )
    }

    #[test]
    fn parses_valid_markdown_into_posts() {
        let result = parse_sources(vec![sample_source("posts/one.md", &valid_markdown("one"))])
            .expect("valid markdown should parse");

        assert_eq!(result.posts().len(), 1);
        assert_eq!(result.tags().len(), 1);
        assert_eq!(result.categories().len(), 1);
        assert_eq!(result.posts()[0].slug, "one");
    }

    #[test]
    fn rejects_duplicate_post_slugs() {
        let first = sample_source("posts/one.md", &valid_markdown("dup"));
        let second = sample_source("posts/two.md", &valid_markdown("dup"));

        let err = parse_sources(vec![first, second]).expect_err("duplicate slug must fail");
        assert!(err.contains("Duplicate post slug"));
    }

    #[test]
    fn rejects_missing_required_fields() {
        let markdown = "---\nslug: \"missing-title\"\npublished_at: \"2026-02-18T12:00:00Z\"\nauthor:\n  name: \"Demo\"\n  email: \"demo@example.com\"\ncategory:\n  name: \"Rust\"\n  slug: \"rust\"\nfeatured_image:\n  file_url: \"/assets/logo.png\"\n---\nBody";

        let err = parse_sources(vec![sample_source("posts/invalid.md", markdown)])
            .expect_err("missing title must fail");

        assert!(err.contains("Invalid frontmatter YAML") || err.contains("`title` is required"));
    }

    #[test]
    fn builds_indexes_for_categories_and_tags() {
        let first = sample_source("posts/one.md", &valid_markdown("one"));

        let second_md = "---\ntitle: \"Post Two\"\nslug: \"two\"\npublished_at: \"2026-02-19T12:00:00Z\"\nauthor:\n  name: \"Demo Author\"\n  email: \"demo@example.com\"\ncategory:\n  name: \"Rust\"\n  slug: \"rust\"\ntags:\n  - name: \"WASM\"\n    slug: \"wasm\"\nfeatured_image:\n  file_url: \"/assets/logo.png\"\n---\nBody";
        let second = sample_source("posts/two.md", second_md);

        let parsed = parse_sources(vec![first, second]).expect("must parse");
        assert_eq!(parsed.categories().len(), 1);
        assert_eq!(parsed.tags().len(), 2);

        let rust = parsed
            .categories()
            .iter()
            .find(|cat| cat.slug == "rust")
            .expect("category exists");

        let rust_posts = parsed.posts_by_category_id(rust.id);
        assert_eq!(rust_posts.len(), 2);

        let dioxus = parsed
            .tags()
            .iter()
            .find(|tag| tag.slug == "dioxus")
            .expect("tag exists");

        let dioxus_posts = parsed.posts_by_tag_id(dioxus.id);
        assert_eq!(dioxus_posts.len(), 1);
    }
}
