use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable, Selectable, Identifiable, Associations, Debug, Clone, Serialize, Deserialize,
)]
#[diesel(table_name = crate::schema::categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub logo_image: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::categories)]
pub struct NewCategory {
    pub name: String,
    pub slug: String,
    pub parent_id: Option<i32>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub logo_image: Option<String>,
}
