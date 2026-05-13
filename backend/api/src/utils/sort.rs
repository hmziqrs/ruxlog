use sea_orm::Order;
use serde::{Deserialize, Serialize};

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
        _ => serializer.serialize_str("desc"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_order_returns_desc() {
        assert_eq!(default_order(), Order::Desc);
    }

    #[test]
    fn test_deserialize_order_lowercase() {
        let val: SortParam = serde_json::from_str(r#"{"field": "created_at", "order": "asc"}"#).unwrap();
        assert_eq!(val.order, Order::Asc);

        let val: SortParam = serde_json::from_str(r#"{"field": "created_at", "order": "desc"}"#).unwrap();
        assert_eq!(val.order, Order::Desc);
    }

    #[test]
    fn test_deserialize_order_uppercase() {
        let val: SortParam = serde_json::from_str(r#"{"field": "created_at", "order": "ASC"}"#).unwrap();
        assert_eq!(val.order, Order::Asc);

        let val: SortParam = serde_json::from_str(r#"{"field": "created_at", "order": "DESC"}"#).unwrap();
        assert_eq!(val.order, Order::Desc);
    }

    #[test]
    fn test_deserialize_order_mixed_case() {
        let val: SortParam = serde_json::from_str(r#"{"field": "created_at", "order": "Asc"}"#).unwrap();
        assert_eq!(val.order, Order::Asc);

        let val: SortParam = serde_json::from_str(r#"{"field": "created_at", "order": "Desc"}"#).unwrap();
        assert_eq!(val.order, Order::Desc);
    }

    #[test]
    fn test_deserialize_order_invalid() {
        let result: Result<SortParam, _> = serde_json::from_str(r#"{"field": "created_at", "order": "invalid"}"#);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid order"));
        assert!(err.contains("invalid"));
    }

    #[test]
    fn test_default_order_when_missing() {
        let val: SortParam = serde_json::from_str(r#"{"field": "created_at"}"#).unwrap();
        assert_eq!(val.order, Order::Desc);
    }

    #[test]
    fn test_serialize_order() {
        let asc = SortParam {
            field: "name".to_string(),
            order: Order::Asc,
        };
        let json = serde_json::to_string(&asc).unwrap();
        assert!(json.contains("\"order\":\"asc\""));

        let desc = SortParam {
            field: "name".to_string(),
            order: Order::Desc,
        };
        let json = serde_json::to_string(&desc).unwrap();
        assert!(json.contains("\"order\":\"desc\""));
    }

    #[test]
    fn test_roundtrip_asc() {
        let val = SortParam {
            field: "title".to_string(),
            order: Order::Asc,
        };
        let json = serde_json::to_string(&val).unwrap();
        let back: SortParam = serde_json::from_str(&json).unwrap();
        assert_eq!(back.field, "title");
        assert_eq!(back.order, Order::Asc);
    }

    #[test]
    fn test_roundtrip_desc() {
        let val = SortParam {
            field: "updated_at".to_string(),
            order: Order::Desc,
        };
        let json = serde_json::to_string(&val).unwrap();
        let back: SortParam = serde_json::from_str(&json).unwrap();
        assert_eq!(back.field, "updated_at");
        assert_eq!(back.order, Order::Desc);
    }

    #[test]
    fn test_various_field_names() {
        let cases = [
            ("id", "id"),
            ("created_at", "created_at"),
            ("title", "title"),
            ("slug", "slug"),
        ];
        for (field_name, expected) in cases {
            let val: SortParam =
                serde_json::from_str(&format!(r#"{{"field": "{}", "order": "asc"}}"#, field_name))
                    .unwrap();
            assert_eq!(val.field, expected);
        }
    }
}
