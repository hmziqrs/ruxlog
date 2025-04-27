use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct NewTag {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TagQuery {
    pub page_no: Option<i64>,
    pub search: Option<String>,
    pub sort_order: Option<String>,
}
