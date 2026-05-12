use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "audit_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub user_id: Option<i32>,
    /// Action performed (e.g., "user.login", "post.create", "plan.update")
    pub action: String,
    /// Type of resource affected (e.g., "user", "post", "plan")
    pub resource_type: String,
    /// ID of the affected resource (string to support various ID types)
    pub resource_id: String,
    /// Arbitrary context about the action
    pub metadata: Option<Json>,
    /// IP address of the actor (supports IPv6)
    pub ip_address: Option<String>,
    /// User agent of the actor
    pub user_agent: Option<String>,
    /// When the audit log entry was created
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::UserId",
        to = "super::super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    User,
}

impl Related<super::super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
