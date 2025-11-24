use crate::db::sea_models::route_status::{BlockFilter, RouteStatusQuery};
use crate::utils::sort::SortParam;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1BlockRoutePayload {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Route pattern must be between 1 and 255 characters"
    ))]
    pub pattern: String,

    #[validate(length(max = 500, message = "Reason must be less than 500 characters"))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdateRoutePayload {
    pub is_blocked: bool,

    #[validate(length(max = 500, message = "Reason must be less than 500 characters"))]
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct V1RouteStatusQueryParams {
    pub page: Option<u64>,
    pub block_filter: Option<BlockFilter>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at_lt: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub updated_at_gt: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub updated_at_lt: Option<chrono::DateTime<chrono::FixedOffset>>,
}

impl V1RouteStatusQueryParams {
    pub fn into_route_status_query(self) -> RouteStatusQuery {
        let block_filter = BlockFilter::resolve(self.block_filter);

        RouteStatusQuery {
            page: self.page,
            block_filter: Some(block_filter),
            search: self.search,
            sorts: self.sorts,
            created_at_gt: self.created_at_gt,
            created_at_lt: self.created_at_lt,
            updated_at_gt: self.updated_at_gt,
            updated_at_lt: self.updated_at_lt,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct V1UpdateSyncIntervalPayload {
    #[validate(range(
        min = 60,
        max = 86400,
        message = "Interval must be between 60 and 86400 seconds"
    ))]
    pub interval_secs: u64,
}
