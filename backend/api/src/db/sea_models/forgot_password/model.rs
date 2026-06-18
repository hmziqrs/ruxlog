use chrono::{Duration, Utc};
use rand::{distr::Alphanumeric, Rng};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "forgot_passwords")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    /// `HMAC-SHA256(secret, code)` — never the plaintext code. See
    /// `utils::code_hash`. Looked up deterministically by hash.
    pub code_hash: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::user::Entity",
        from = "Column::UserId",
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
    pub const DELAY_TIME: Duration = Duration::minutes(1);
    pub const EXPIRY_TIME: Duration = Duration::hours(3);

    /// Generate a fresh plaintext code. 8 chars over the full alphanumeric
    /// alphabet (mixed case) ≈ 47 bits of entropy — enough to defeat online
    /// guessing under the Phase-3c rate limiter. Returned in plaintext to the
    /// caller, which emails it and stores only `hash_code(secret, code)`.
    pub fn generate_code() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect()
    }
}

impl Model {
    pub fn is_expired(&self) -> bool {
        Utc::now().fixed_offset() > self.updated_at + Entity::EXPIRY_TIME
    }

    pub fn is_in_delay(&self) -> bool {
        let delay_time = self.updated_at + Entity::DELAY_TIME;
        Utc::now().fixed_offset() < delay_time
    }
}
