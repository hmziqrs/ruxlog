use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Desc
    }
}

fn serialize_order<S>(order: &SortOrder, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(match order {
        SortOrder::Asc => "asc",
        SortOrder::Desc => "desc",
    })
}

fn deserialize_order<'de, D>(d: D) -> Result<SortOrder, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    match s.to_lowercase().as_str() {
        "asc" => Ok(SortOrder::Asc),
        _ => Ok(SortOrder::Desc),
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
    pub order: SortOrder,
}

fn default_order() -> SortOrder {
    SortOrder::Desc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_order_default_is_desc() {
        assert_eq!(SortOrder::default(), SortOrder::Desc);
    }

    #[test]
    fn sort_param_serde_roundtrip() {
        let param = SortParam {
            field: "created_at".to_string(),
            order: SortOrder::Asc,
        };
        let json = serde_json::to_string(&param).unwrap();
        let parsed: SortParam = serde_json::from_str(&json).unwrap();
        assert_eq!(param.field, parsed.field);
        assert_eq!(param.order, parsed.order);
    }

    #[test]
    fn sort_param_deserialize_lowercase() {
        let json = r#"{"field":"title","order":"desc"}"#;
        let parsed: SortParam = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.order, SortOrder::Desc);
    }
}
