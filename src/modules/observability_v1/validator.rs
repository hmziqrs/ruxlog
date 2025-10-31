use chrono::{Duration, Utc};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct V1LogsSearchPayload {
    #[validate(length(min = 1, max = 1000))]
    pub sql: Option<String>,

    pub start_time: Option<i64>,
    pub end_time: Option<i64>,

    #[validate(range(min = 0, max = 10000))]
    pub from: Option<i64>,

    #[validate(range(min = 1, max = 1000))]
    pub size: Option<i64>,

    #[validate(length(min = 1, max = 100))]
    pub stream: Option<String>,
}

impl V1LogsSearchPayload {
    pub fn get_sql(&self) -> String {
        self.sql
            .clone()
            .unwrap_or_else(|| "SELECT * FROM {stream} ORDER BY _timestamp DESC".to_string())
    }

    pub fn get_stream(&self) -> String {
        self.stream.clone().unwrap_or_else(|| "default".to_string())
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
    pub fn build_sql(&self) -> String {
        let mut conditions = vec![];

        if let Some(ref level) = self.level {
            conditions.push(format!("level = '{}'", level));
        }

        if let Some(ref service) = self.service {
            conditions.push(format!("service_name = '{}'", service));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        format!(
            "SELECT * FROM {{stream}}{} ORDER BY _timestamp DESC",
            where_clause
        )
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

    pub fn build_sql(&self) -> String {
        if let Some(ref metric) = self.metric_name {
            format!(
                "SELECT histogram(_timestamp, '5 minute') AS time_bucket, \
                 COUNT(*) AS count, \
                 AVG(value) AS avg_value \
                 FROM {{stream}} \
                 WHERE metric_name = '{}' \
                 GROUP BY time_bucket \
                 ORDER BY time_bucket DESC",
                metric
            )
        } else {
            "SELECT metric_name, COUNT(*) AS count \
             FROM {stream} \
             GROUP BY metric_name \
             ORDER BY count DESC"
                .to_string()
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

    pub fn build_sql(&self) -> String {
        let limit = self.top_n.unwrap_or(20);
        format!(
            "SELECT http_route, http_method, COUNT(*) AS error_count \
             FROM {{stream}} \
             WHERE level = 'ERROR' OR http_status_code >= 400 \
             GROUP BY http_route, http_method \
             ORDER BY error_count DESC \
             LIMIT {}",
            limit
        )
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

    pub fn build_sql(&self) -> String {
        let route_filter = if let Some(ref route) = self.route {
            format!(" WHERE http_route = '{}'", route)
        } else {
            String::new()
        };

        format!(
            "SELECT http_route, \
             COUNT(*) AS request_count, \
             AVG(duration_ms) AS avg_latency_ms, \
             MIN(duration_ms) AS min_latency_ms, \
             MAX(duration_ms) AS max_latency_ms \
             FROM {{stream}}{} \
             GROUP BY http_route \
             ORDER BY request_count DESC \
             LIMIT 50",
            route_filter
        )
    }
}
