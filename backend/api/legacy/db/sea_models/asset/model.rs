use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "asset_context")]
#[serde(rename_all = "kebab-case")]
pub enum AssetContext {
    #[sea_orm(string_value = "user-avatar")]
    UserAvatar,
    #[sea_orm(string_value = "category-cover")]
    CategoryCover,
    #[sea_orm(string_value = "category-logo")]
    CategoryLogo,
    #[sea_orm(string_value = "post-featured")]
    PostFeatured,
    #[sea_orm(string_value = "post-inline")]
    PostInline,
}

impl AssetContext {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetContext::UserAvatar => "user-avatar",
            AssetContext::CategoryCover => "category-cover",
            AssetContext::CategoryLogo => "category-logo",
            AssetContext::PostFeatured => "post-featured",
            AssetContext::PostInline => "post-inline",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "user-avatar" => Ok(AssetContext::UserAvatar),
            "category-cover" => Ok(AssetContext::CategoryCover),
            "category-logo" => Ok(AssetContext::CategoryLogo),
            "post-featured" => Ok(AssetContext::PostFeatured),
            "post-inline" => Ok(AssetContext::PostInline),
            _ => Err(format!("Invalid asset context: {}", s)),
        }
    }
}

impl std::str::FromStr for AssetContext {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AssetContext::from_str(s)
    }
}

impl From<&str> for AssetContext {
    fn from(s: &str) -> Self {
        AssetContext::from_str(s).unwrap_or(AssetContext::UserAvatar)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "assets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub file_url: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i32>,
    pub uploaded_at: DateTimeWithTimeZone,
    pub owner_id: Option<i32>,
    pub context: Option<AssetContext>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::OwnerId",
        to = "super::super::user::Column::Id"
    )]
    User,
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn find_by_owner(db: &DbConn, owner_id: i32) -> Result<Vec<Model>, DbErr> {
        Self::find()
            .filter(Column::OwnerId.eq(owner_id))
            .all(db)
            .await
    }

    pub async fn find_by_context(db: &DbConn, context: AssetContext) -> Result<Vec<Model>, DbErr> {
        Self::find()
            .filter(Column::Context.eq(context))
            .all(db)
            .await
    }
}

impl Model {
    pub fn human_readable_size(&self) -> String {
        match self.size {
            Some(size) => {
                if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.2} KB", size as f64 / 1024.0)
                } else if size < 1024 * 1024 * 1024 {
                    format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                }
            }
            None => "Unknown size".to_string(),
        }
    }

    pub fn get_extension(&self) -> Option<String> {
        if let Some(ref file_name) = self.file_name {
            let parts: Vec<&str> = file_name.split('.').collect();
            if parts.len() > 1 {
                return Some(parts.last().unwrap().to_string());
            }
        }

        if let Some(ref mime) = self.mime_type {
            let parts: Vec<&str> = mime.split('/').collect();
            if parts.len() > 1 {
                return Some(parts.last().unwrap().to_string());
            }
        }

        None
    }
}
