use serde::{Deserialize, Serialize};

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(feature = "backend", sea_orm(rs_type = "String", db_type = "Text"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubscriberStatus {
    #[serde(rename = "Pending")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "pending"))]
    Pending,
    #[serde(rename = "Confirmed")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "confirmed"))]
    Confirmed,
    #[serde(rename = "Unsubscribed")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "unsubscribed"))]
    Unsubscribed,
}
