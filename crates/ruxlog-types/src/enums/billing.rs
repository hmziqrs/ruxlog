use serde::{Deserialize, Serialize};

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "plan_interval")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanInterval {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "monthly"))]
    Monthly,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "yearly"))]
    Yearly,
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(
        rs_type = "String",
        db_type = "Enum",
        enum_name = "subscription_status"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "active"))]
    Active,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "past_due"))]
    PastDue,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "canceled"))]
    Canceled,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "expired"))]
    Expired,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "trialing"))]
    Trialing,
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "payment_status")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "pending"))]
    Pending,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "completed"))]
    Completed,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "failed"))]
    Failed,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "refunded"))]
    Refunded,
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "discount_type")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscountType {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "percentage"))]
    Percentage,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "fixed_amount"))]
    FixedAmount,
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "invoice_status")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "draft"))]
    Draft,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "sent"))]
    Sent,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "paid"))]
    Paid,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "void"))]
    Void,
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(
        rs_type = "String",
        db_type = "Enum",
        enum_name = "payout_account_status"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutAccountStatus {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "pending"))]
    Pending,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "verified"))]
    Verified,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "rejected"))]
    Rejected,
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ledger_entry_type")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerEntryType {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "credit"))]
    Credit,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "debit"))]
    Debit,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "payout"))]
    Payout,
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(
        rs_type = "String",
        db_type = "Enum",
        enum_name = "scheduled_post_status"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScheduledPostStatus {
    #[serde(rename = "Pending")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "pending"))]
    Pending,
    #[serde(rename = "Published")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "published"))]
    Published,
    #[serde(rename = "Canceled")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "canceled"))]
    Canceled,
    #[serde(rename = "Failed")]
    #[cfg_attr(feature = "backend", sea_orm(string_value = "failed"))]
    Failed,
}

impl std::fmt::Display for ScheduledPostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Published => "published",
            Self::Canceled => "canceled",
            Self::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

#[cfg_attr(
    feature = "backend",
    derive(sea_orm::DeriveActiveEnum, strum::EnumIter)
)]
#[cfg_attr(
    feature = "backend",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "post_access_type")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostAccessType {
    #[cfg_attr(feature = "backend", sea_orm(string_value = "free"))]
    Free,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "paid"))]
    Paid,
    #[cfg_attr(feature = "backend", sea_orm(string_value = "subscriber_only"))]
    SubscriberOnly,
}
