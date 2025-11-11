use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sorting order for query parameters
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl Default for Order {
    fn default() -> Self {
        Order::Desc
    }
}

/// Sort parameter with field name and order direction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SortParam {
    pub field: String,
    #[serde(
        default = "default_order",
        deserialize_with = "deserialize_order",
        serialize_with = "serialize_order"
    )]
    pub order: Order,
}

impl Default for SortParam {
    fn default() -> Self {
        Self {
            field: String::new(),
            order: Order::default(),
        }
    }
}

fn default_order() -> Order {
    Order::Desc
}

fn serialize_order<S>(order: &Order, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match order {
        Order::Asc => serializer.serialize_str("asc"),
        Order::Desc => serializer.serialize_str("desc"),
    }
}

fn deserialize_order<'de, D>(deserializer: D) -> Result<Order, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "asc" | "ASC" | "Asc" => Ok(Order::Asc),
        "desc" | "DESC" | "Desc" => Ok(Order::Desc),
        other => Err(serde::de::Error::custom(format!(
            "invalid order '{}', expected 'asc' or 'desc'",
            other
        ))),
    }
}

/// Trait for list query parameters
pub trait ListQuery: Clone + Default + Serialize + for<'de> Deserialize<'de> + PartialEq {
    fn new() -> Self;

    fn page(&self) -> u64;

    fn set_page(&mut self, page: u64);

    fn search(&self) -> Option<String>;

    fn set_search(&mut self, search: Option<String>);

    fn sorts(&self) -> Option<Vec<SortParam>>;

    fn set_sorts(&mut self, sorts: Option<Vec<SortParam>>);
}

/// Base structure for common list query fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseListQuery {
    pub page: u64,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    pub updated_at_gt: Option<DateTime<Utc>>,
    pub updated_at_lt: Option<DateTime<Utc>>,
}

impl Default for BaseListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            search: None,
            sorts: None,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
        }
    }
}

impl BaseListQuery {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ListQuery for BaseListQuery {
    fn new() -> Self {
        Self::new()
    }

    fn page(&self) -> u64 {
        self.page
    }

    fn set_page(&mut self, page: u64) {
        self.page = page;
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