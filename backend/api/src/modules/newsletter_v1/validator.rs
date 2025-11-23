use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{db::sea_models::newsletter_subscriber::SubscriberQuery, utils::SortParam};

/// Subscribe to newsletter (double opt-in)
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1SubscribePayload {
    #[validate(email)]
    pub email: String,
}

/// Unsubscribe from newsletter
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UnsubscribePayload {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, max = 128))]
    pub token: String,
}
/// Confirm newsletter subscription (same as unsubscribe payload)
pub type V1ConfirmPayload = V1UnsubscribePayload;

/// Send a newsletter (admin)
#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1SendNewsletterPayload {
    #[validate(length(min = 1, max = 200))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub text: String,
    pub html: Option<String>,
}

/// List subscribers (admin) with optional pagination and search
#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct V1ListSubscribersQuery {
    pub page: Option<u64>,
    #[validate(length(min = 1, max = 100))]
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}

impl V1ListSubscribersQuery {
    pub fn page_or_default(&self) -> u64 {
        self.page.unwrap_or(1)
    }

    pub fn into_query(self) -> SubscriberQuery {
        SubscriberQuery {
            page: self.page,
            search: self.search,
            status: None,
            sorts: self.sorts,
            created_at_gt: self.created_at_gt,
            created_at_lt: self.created_at_lt,
            updated_at_gt: self.updated_at_gt,
            updated_at_lt: self.updated_at_lt,
        }
    }
}
