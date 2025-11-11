use crate::store::{ListQuery, Media, PaginatedList, StateFrame};
use crate::types::SortParam;
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// Core Post Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostAuthor {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub avatar: Option<Media>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostCategory {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub color: String,
    #[serde(default)]
    pub logo: Option<Media>,
    #[serde(default)]
    pub cover: Option<Media>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostTag {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

impl fmt::Display for PostStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            PostStatus::Draft => "Draft",
            PostStatus::Published => "Published",
            PostStatus::Archived => "Archived",
        };
        f.write_str(label)
    }
}

impl Default for PostStatus {
    fn default() -> Self {
        PostStatus::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PostContent {
    pub time: u64,
    pub blocks: Vec<EditorJsBlock>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum EditorJsBlock {
    #[serde(rename = "header")]
    Header {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: HeaderBlock,
    },
    #[serde(rename = "paragraph")]
    Paragraph {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: ParagraphBlock,
    },
    #[serde(rename = "list")]
    List {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: ListBlock,
    },
    #[serde(rename = "delimiter")]
    Delimiter {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    #[serde(rename = "image")]
    Image {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: ImageBlock,
    },
    #[serde(rename = "embed")]
    Embed {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: EmbedBlock,
    },
    #[serde(rename = "linktool")]
    LinkTool {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: LinkToolBlock,
    },
    #[serde(rename = "attaches")]
    Attaches {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: AttachesBlock,
    },
    #[serde(rename = "code")]
    Code {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: CodeBlock,
    },
    #[serde(rename = "raw")]
    Raw {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: RawBlock,
    },
    #[serde(rename = "table")]
    Table {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: TableBlock,
    },
    #[serde(rename = "quote")]
    Quote {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: QuoteBlock,
    },
    #[serde(rename = "warning")]
    Warning {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: WarningBlock,
    },
    #[serde(rename = "button")]
    Button {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: ButtonBlock,
    },
    #[serde(rename = "alert")]
    Alert {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: AlertBlock,
    },
    #[serde(rename = "checklist")]
    Checklist {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        data: ChecklistBlock,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeaderBlock {
    pub text: String,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParagraphBlock {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeBlock {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuoteBlock {
    pub text: String,
    pub caption: Option<String>,
    pub alignment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertBlock {
    #[serde(rename = "type")]
    pub alert_type: String,
    pub align: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChecklistItem {
    pub text: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChecklistBlock {
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListBlock {
    pub style: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageBlock {
    pub file: ImageFile,
    pub caption: Option<String>,
    pub stretched: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub withBackground: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub withBorder: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageFile {
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub name: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub media_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbedBlock {
    pub embed: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub caption: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkToolBlock {
    pub link: String,
    pub meta: LinkToolMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkToolMeta {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<ImageFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachesBlock {
    pub file: AttachesFile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachesFile {
    pub url: String,
    pub size: i64,
    pub name: String,
    pub extension: String,
    #[serde(default)]
    pub media_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawBlock {
    pub html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableBlock {
    pub content: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WarningBlock {
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ButtonBlock {
    pub text: String,
    pub link: Option<String>,
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: PostContent,
    pub slug: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<Media>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: PostAuthor,
    pub category: PostCategory,
    #[serde(default)]
    pub tags: Vec<PostTag>,
    pub likes_count: i32,
    pub view_count: i32,
    pub comment_count: i64,
    pub status: PostStatus,
}

impl Post {
    pub fn is_published(&self) -> bool {
        self.status == PostStatus::Published
    }

    pub fn is_draft(&self) -> bool {
        self.status == PostStatus::Draft
    }

    pub fn is_archived(&self) -> bool {
        self.status == PostStatus::Archived
    }
}

// ============================================================================
// Post Payloads (matching backend V1 API)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PostCreatePayload {
    pub title: String,
    pub content: PostContent,
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_published: bool,
    pub slug: String,
    pub excerpt: Option<String>,
    pub featured_image_id: Option<i32>,
    pub category_id: i32,
    #[serde(default)]
    pub tag_ids: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PostEditPayload {
    pub title: Option<String>,
    pub content: Option<PostContent>,
    pub published_at: Option<DateTime<Utc>>,
    pub status: Option<PostStatus>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub featured_image_id: Option<i32>,
    pub category_id: Option<i32>,
    pub tag_ids: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PostListQuery {
    pub page: Option<u64>,
    pub author_id: Option<i32>,
    pub category_id: Option<i32>,
    pub status: Option<PostStatus>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub tag_ids: Option<Vec<i32>>,
    pub title: Option<String>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    pub updated_at_gt: Option<DateTime<Utc>>,
    pub updated_at_lt: Option<DateTime<Utc>>,
    pub published_at_gt: Option<DateTime<Utc>>,
    pub published_at_lt: Option<DateTime<Utc>>,
}

impl PostListQuery {
    pub fn new() -> Self {
        Self {
            page: Some(1),
            ..Default::default()
        }
    }
}

impl ListQuery for PostListQuery {
    fn new() -> Self {
        Self::new()
    }

    fn page(&self) -> u64 {
        self.page.unwrap_or(1)
    }

    fn set_page(&mut self, page: u64) {
        self.page = Some(page);
    }

    fn search(&self) -> Option<String> {
        self.search.clone()
    }

    fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }

    fn sorts(&self) -> Option<Vec<SortParam>> {
        self.sorts.clone()
    }

    fn set_sorts(&mut self, sorts: Option<Vec<SortParam>>) {
        self.sorts = sorts;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostSchedulePayload {
    pub post_id: i32,
    pub publish_at: DateTime<Utc>,
}

// ============================================================================
// Post Revisions
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PostRevision {
    pub id: i32,
    pub post_id: i32,
    pub title: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: i32,
}

// ============================================================================
// Series Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Series {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SeriesCreatePayload {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SeriesEditPayload {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SeriesListQuery {
    pub page: Option<u64>,
    pub search: Option<String>,
}

impl SeriesListQuery {
    pub fn new() -> Self {
        Self {
            page: Some(1),
            ..Default::default()
        }
    }
}

impl ListQuery for SeriesListQuery {
    fn new() -> Self {
        Self::new()
    }

    fn page(&self) -> u64 {
        self.page.unwrap_or(1)
    }

    fn set_page(&mut self, page: u64) {
        self.page = Some(page);
    }

    fn search(&self) -> Option<String> {
        self.search.clone()
    }

    fn set_search(&mut self, search: Option<String>) {
        self.search = search;
    }

    fn sorts(&self) -> Option<Vec<SortParam>> {
        None
    }

    fn set_sorts(&mut self, _sorts: Option<Vec<SortParam>>) {
        // Series list doesn't support custom sorts
    }
}

// ============================================================================
// State Management
// ============================================================================

pub struct PostState {
    // Core CRUD operations
    pub view: GlobalSignal<HashMap<i32, StateFrame<Post>>>, // Keyed by post id
    pub list: GlobalSignal<StateFrame<PaginatedList<Post>>>,
    pub add: GlobalSignal<StateFrame<Post, PostCreatePayload>>,
    pub edit: GlobalSignal<HashMap<i32, StateFrame<(), PostEditPayload>>>,
    pub remove: GlobalSignal<HashMap<i32, StateFrame>>,

    // Scheduling
    pub schedule: GlobalSignal<HashMap<i32, StateFrame>>,

    // Revisions
    pub revisions_list: GlobalSignal<HashMap<i32, StateFrame<Vec<PostRevision>>>>,
    pub revisions_restore: GlobalSignal<HashMap<(i32, i32), StateFrame>>, // (post_id, revision_id)

    // View tracking
    pub track_view: GlobalSignal<HashMap<i32, StateFrame>>,

    // Series management
    pub series_list: GlobalSignal<StateFrame<PaginatedList<Series>>>,
    pub series_view: GlobalSignal<HashMap<i32, StateFrame<Series>>>,
    pub series_add: GlobalSignal<StateFrame<(), SeriesCreatePayload>>,
    pub series_edit: GlobalSignal<HashMap<i32, StateFrame<(), SeriesEditPayload>>>,
    pub series_remove: GlobalSignal<HashMap<i32, StateFrame>>,
    pub series_add_post: GlobalSignal<HashMap<(i32, i32), StateFrame>>, // (post_id, series_id)
    pub series_remove_post: GlobalSignal<HashMap<(i32, i32), StateFrame>>, // (post_id, series_id)
}

impl PostState {
    pub fn new() -> Self {
        PostState {
            view: GlobalSignal::new(|| HashMap::new()),
            list: GlobalSignal::new(|| StateFrame::new()),
            add: GlobalSignal::new(|| StateFrame::new()),
            edit: GlobalSignal::new(|| HashMap::new()),
            remove: GlobalSignal::new(|| HashMap::new()),
            schedule: GlobalSignal::new(|| HashMap::new()),
            revisions_list: GlobalSignal::new(|| HashMap::new()),
            revisions_restore: GlobalSignal::new(|| HashMap::new()),
            track_view: GlobalSignal::new(|| HashMap::new()),
            series_list: GlobalSignal::new(|| StateFrame::new()),
            series_view: GlobalSignal::new(|| HashMap::new()),
            series_add: GlobalSignal::new(|| StateFrame::new()),
            series_edit: GlobalSignal::new(|| HashMap::new()),
            series_remove: GlobalSignal::new(|| HashMap::new()),
            series_add_post: GlobalSignal::new(|| HashMap::new()),
            series_remove_post: GlobalSignal::new(|| HashMap::new()),
        }
    }
}

static POST_STATE: std::sync::OnceLock<PostState> = std::sync::OnceLock::new();

pub fn use_post() -> &'static PostState {
    POST_STATE.get_or_init(|| PostState::new())
}
