use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

use crate::utils::sort::SortParam;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BlockFilter {
    All,
    Blocked,
    Unblocked,
}

impl BlockFilter {
    pub fn resolve(input: Option<Self>) -> Self {
        input.unwrap_or(BlockFilter::All)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteStatusQuery {
    pub page: Option<u64>,
    pub block_filter: Option<BlockFilter>,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<DateTimeWithTimeZone>,
    pub created_at_lt: Option<DateTimeWithTimeZone>,
    pub updated_at_gt: Option<DateTimeWithTimeZone>,
    pub updated_at_lt: Option<DateTimeWithTimeZone>,
}
