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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Order serde (default enum variant names) ──

    #[test]
    fn order_serialize_uses_variant_name() {
        assert_eq!(serde_json::to_string(&Order::Asc).unwrap(), "\"Asc\"");
        assert_eq!(serde_json::to_string(&Order::Desc).unwrap(), "\"Desc\"");
    }

    #[test]
    fn order_deserialize_uses_variant_name() {
        let asc: Order = serde_json::from_str("\"Asc\"").unwrap();
        assert_eq!(asc, Order::Asc);
        let desc: Order = serde_json::from_str("\"Desc\"").unwrap();
        assert_eq!(desc, Order::Desc);
    }

    #[test]
    fn order_deserialize_invalid_errors() {
        let result: Result<Order, _> = serde_json::from_str("\"random\"");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unknown variant"));
    }

    // ── Order serde in SortParam uses custom serializer (lowercase) ──

    #[test]
    fn sort_param_order_serializes_lowercase() {
        let sp = SortParam {
            field: "created_at".into(),
            order: Order::Asc,
        };
        let json = serde_json::to_string(&sp).unwrap();
        assert!(json.contains("\"asc\""));
        assert!(!json.contains("\"Asc\""));
    }

    #[test]
    fn sort_param_order_deserializes_case_insensitive() {
        // lowercase
        let sp: SortParam =
            serde_json::from_str(r#"{"field":"name","order":"asc"}"#).unwrap();
        assert_eq!(sp.order, Order::Asc);
        // uppercase
        let sp: SortParam =
            serde_json::from_str(r#"{"field":"name","order":"ASC"}"#).unwrap();
        assert_eq!(sp.order, Order::Asc);
        // mixed
        let sp: SortParam =
            serde_json::from_str(r#"{"field":"name","order":"Desc"}"#).unwrap();
        assert_eq!(sp.order, Order::Desc);
    }

    #[test]
    fn sort_param_order_invalid_errors() {
        let result: Result<SortParam, _> =
            serde_json::from_str(r#"{"field":"name","order":"random"}"#);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid order"));
    }

    // ── SortParam defaults ──

    #[test]
    fn sort_param_default() {
        let sp = SortParam::default();
        assert_eq!(sp.field, "");
        assert_eq!(sp.order, Order::Desc);
    }

    #[test]
    fn sort_param_serialize_roundtrip() {
        let sp = SortParam {
            field: "created_at".into(),
            order: Order::Asc,
        };
        let json = serde_json::to_string(&sp).unwrap();
        let parsed: SortParam = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.field, "created_at");
        assert_eq!(parsed.order, Order::Asc);
    }

    // ── BaseListQuery defaults ──

    #[test]
    fn base_list_query_default_page_is_one() {
        let q = BaseListQuery::default();
        assert_eq!(q.page, 1);
    }

    #[test]
    fn base_list_query_new_matches_default() {
        let q = BaseListQuery::new();
        assert_eq!(q.page, 1);
        assert!(q.search.is_none());
        assert!(q.sorts.is_none());
    }

    #[test]
    fn base_list_query_list_query_trait_impl() {
        let mut q = BaseListQuery::new();
        assert_eq!(q.page(), 1);

        q.set_page(3);
        assert_eq!(q.page(), 3);

        assert!(q.search().is_none());
        q.set_search(Some("hello".into()));
        assert_eq!(q.search(), Some("hello".into()));

        assert!(q.sorts().is_none());
        q.set_sorts(Some(vec![SortParam {
            field: "name".into(),
            order: Order::Asc,
        }]));
        assert!(q.sorts().is_some());
    }
}