use chrono::{Duration, Utc};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct V1LogsSearchPayload {
    #[validate(length(min = 1, max = 1000))]
    pub query: Option<String>,

    pub start_time: Option<i64>,
    pub end_time: Option<i64>,

    #[validate(range(min = 0, max = 10000))]
    pub from: Option<i64>,

    #[validate(range(min = 1, max = 1000))]
    pub size: Option<i64>,

    #[validate(length(min = 1, max = 100))]
    pub index: Option<String>,
}

impl V1LogsSearchPayload {
    pub fn get_query(&self) -> String {
        self.query
            .clone()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "*".to_string())
    }

    pub fn get_index(&self) -> Option<String> {
        self.index
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
    }

    pub fn get_time_range(&self) -> (i64, i64) {
        let now = Utc::now();
        let end = self.end_time.unwrap_or_else(|| now.timestamp_micros());
        let start = self
            .start_time
            .unwrap_or_else(|| (now - Duration::hours(1)).timestamp_micros());
        (start, end)
    }

    pub fn get_from(&self) -> i64 {
        self.from.unwrap_or(0)
    }

    pub fn get_size(&self) -> i64 {
        self.size.unwrap_or(100)
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct V1LogsRecentPayload {
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<i64>,

    #[validate(length(min = 1, max = 50))]
    pub level: Option<String>,

    #[validate(length(min = 1, max = 100))]
    pub service: Option<String>,

    pub hours_ago: Option<i64>,
}

impl V1LogsRecentPayload {
    pub fn build_query(&self) -> String {
        let mut conditions = vec![];

        if let Some(ref level) = self.level {
            conditions.push(format!("level:\"{}\"", escape_term(level)));
        }

        if let Some(ref service) = self.service {
            conditions.push(format!("service_name:\"{}\"", escape_term(service)));
        }

        if conditions.is_empty() {
            "*".to_string()
        } else {
            conditions.join(" AND ")
        }
    }

    pub fn get_time_range(&self) -> (i64, i64) {
        let now = Utc::now();
        let hours = self.hours_ago.unwrap_or(1);
        let start = (now - Duration::hours(hours)).timestamp_micros();
        let end = now.timestamp_micros();
        (start, end)
    }

    pub fn get_limit(&self) -> i64 {
        self.limit.unwrap_or(100)
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct V1MetricsSummaryPayload {
    pub hours_ago: Option<i64>,

    #[validate(length(min = 1, max = 100))]
    pub metric_name: Option<String>,
}

impl V1MetricsSummaryPayload {
    pub fn get_time_range(&self) -> (i64, i64) {
        let now = Utc::now();
        let hours = self.hours_ago.unwrap_or(24);
        let start = (now - Duration::hours(hours)).timestamp_micros();
        let end = now.timestamp_micros();
        (start, end)
    }

    pub fn build_query(&self) -> String {
        if let Some(ref metric) = self.metric_name {
            format!("metric_name:\"{}\"", escape_term(metric))
        } else {
            "*".to_string()
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct V1ErrorStatsPayload {
    pub hours_ago: Option<i64>,

    #[validate(range(min = 1, max = 100))]
    pub top_n: Option<i64>,
}

impl V1ErrorStatsPayload {
    pub fn get_time_range(&self) -> (i64, i64) {
        let now = Utc::now();
        let hours = self.hours_ago.unwrap_or(24);
        let start = (now - Duration::hours(hours)).timestamp_micros();
        let end = now.timestamp_micros();
        (start, end)
    }

    pub fn build_query(&self) -> String {
        let base_filter = "(level:ERROR OR http_status_code:[400 TO *])";
        base_filter.to_string()
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct V1LatencyStatsPayload {
    pub hours_ago: Option<i64>,

    #[validate(length(min = 1, max = 200))]
    pub route: Option<String>,
}

impl V1LatencyStatsPayload {
    pub fn get_time_range(&self) -> (i64, i64) {
        let now = Utc::now();
        let hours = self.hours_ago.unwrap_or(24);
        let start = (now - Duration::hours(hours)).timestamp_micros();
        let end = now.timestamp_micros();
        (start, end)
    }

    pub fn build_query(&self) -> String {
        if let Some(ref route) = self.route {
            format!("http_route:\"{}\"", escape_term(route))
        } else {
            "*".to_string()
        }
    }
}

fn escape_term(value: &str) -> String {
    value.replace('"', "\\\"")
}
