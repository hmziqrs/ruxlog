use chrono::{Duration, NaiveDateTime, Utc};
use rand::{distr::Alphanumeric, Rng};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "email_verifications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub code: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    // Constants for verification timing
    pub const DELAY_TIME: Duration = Duration::minutes(1);
    pub const EXPIRY_TIME: Duration = Duration::hours(3);

    // Generate a random verification code
    pub fn generate_code() -> String {
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect::<String>()
            .to_lowercase()
    }
}

impl Model {
    // Check if a verification code has expired
    pub fn is_expired(&self) -> bool {
        Utc::now().naive_utc() > self.expires_at
    }

    // Check if a verification code is still in the delay period
    pub fn is_in_delay(&self) -> bool {
        let delay_time = self.updated_at + Entity::DELAY_TIME;
        Utc::now().naive_utc() < delay_time
    }

    // Send verification email
    // pub async fn send_email(&self, email: &str) -> Result<(), Error> {
    //     // Implement your email sending logic here
    //     // For example, using an email sending crate like lettre
    //     Ok(())
    // }
}
