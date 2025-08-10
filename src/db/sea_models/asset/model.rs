use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub context: Option<String>,
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
    pub async fn find_by_owner(
        db: &DbConn,
        owner_id: i32,
    ) -> Result<Vec<Model>, DbErr> {
        Self::find()
            .filter(Column::OwnerId.eq(owner_id))
            .all(db)
            .await
    }

    pub async fn find_by_context(
        db: &DbConn,
        context: &str,
    ) -> Result<Vec<Model>, DbErr> {
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