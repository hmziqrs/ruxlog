use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

// Define the post status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "post_status")]
pub enum PostStatus {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "published")]
    Published,
    #[sea_orm(string_value = "archived")]
    Archived,
}

impl fmt::Display for PostStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Published => write!(f, "published"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

// Define the entity for 'posts' table
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: Option<String>,
    pub featured_image: Option<String>,
    pub status: PostStatus,
    pub published_at: Option<NaiveDateTime>,

    pub author_id: i32,
    pub category_id: Option<i32>,
    pub view_count: i32,
    pub likes_count: i32,
    pub tag_ids: Vec<i32>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Define the relations
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::AuthorId",
        to = "super::super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::super::post_comment::Entity")]
    Comment,
    #[sea_orm(has_many = "super::super::post_view::Entity")]
    View,
    #[sea_orm(
        belongs_to = "super::super::category::Entity",
        from = "Column::CategoryId",
        to = "super::super::category::Column::Id"
    )]
    Category,
    // We're using a tag_ids array directly in the Post model for now
    // For a real many-to-many we'd use a join table,
    // but for now, just removing this relation
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::super::post_comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
    }
}

impl Related<super::super::post_view::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::View.def()
    }
}

impl Related<super::super::category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

// Tags are stored as array directly in the post model

// ActiveModel is the mutable version of Model
impl ActiveModelBehavior for ActiveModel {}
