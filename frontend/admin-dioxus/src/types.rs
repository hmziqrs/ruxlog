use serde::{Deserialize, Serialize};

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
