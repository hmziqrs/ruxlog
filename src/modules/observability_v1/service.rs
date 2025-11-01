
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::error;

const DEFAULT_API_URL: &str = "http://localhost:7280";
const DEFAULT_LOGS_INDEX: &str = "otel-logs-v0_7";
const DEFAULT_TRACES_INDEX: &str = "otel-traces-v0_7";
const DEFAULT_METRICS_INDEX: &str = "otel-metrics-v0_7";

#[derive(Clone, Debug)]
pub struct QuickwitConfig {
    pub api_url: String,
    pub logs_index: String,
    pub traces_index: String,
    pub metrics_index: String,
    pub access_token: Option<String>,
    pub enabled: bool,
}

impl QuickwitConfig {
    pub fn from_env() -> Self {
        let api_url = env::var("QUICKWIT_API_URL")
            .unwrap_or_else(|_| DEFAULT_API_URL.to_string())
            .trim_end_matches('/')
            .to_string();

        let logs_index =
            env::var("QUICKWIT_LOGS_INDEX_ID").unwrap_or_else(|_| DEFAULT_LOGS_INDEX.to_string());

        let traces_index = env::var("QUICKWIT_TRACES_INDEX_ID")
            .unwrap_or_else(|_| DEFAULT_TRACES_INDEX.to_string());

        let metrics_index = env::var("QUICKWIT_METRICS_INDEX_ID")
            .unwrap_or_else(|_| DEFAULT_METRICS_INDEX.to_string());

        let access_token = env::var("QUICKWIT_ACCESS_TOKEN").ok();

        let enabled = env::var("ENABLE_QUICKWIT_OTEL")
            .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);

        Self {
            api_url,
            logs_index,
            traces_index,
            metrics_index,
            access_token,
            enabled,
        }
    }
}

#[derive(Clone)]
pub struct QuickwitClient {
    client: Client,
    config: QuickwitConfig,
}

impl QuickwitClient {
    pub fn new(config: QuickwitConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub fn logs_index(&self) -> &str {
        &self.config.logs_index
    }

    pub fn traces_index(&self) -> &str {
        &self.config.traces_index
    }

    pub fn metrics_index(&self) -> &str {
        &self.config.metrics_index
    }

    pub async fn search(
        &self,
        index: Option<&str>,
        query: &str,
        _start_time_micros: i64,
        _end_time_micros: i64,
        offset: i64,
        limit: i64,
    ) -> Result<SearchResponse, QuickwitError> {
        if !self.config.enabled {
            return Err(QuickwitError::Disabled);
        }

        let index = index
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| self.logs_index());
        let url = format!("{}/api/v1/{}/search", self.config.api_url, index);

        let request = SearchRequest {
            query: query.to_string(),
            start_timestamp: None,
            end_timestamp: None,
            max_hits: Some(limit.max(0)),
            start_offset: Some(offset.max(0)),
        };

        let mut builder = self.client.post(&url).json(&request);

        if let Some(token) = &self.config.access_token {
            builder = builder.bearer_auth(token);
        }

        let response = builder.send().await.map_err(|e| {
            error!(error = %e, "Failed to send request to Quickwit");
            QuickwitError::RequestFailed(e.to_string())
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Quickwit API error");
            return Err(QuickwitError::ApiError(status.as_u16(), body));
        }

        let search_response = response.json::<SearchResponse>().await.map_err(|e| {
            error!(error = %e, "Failed to parse Quickwit response");
            QuickwitError::ParseError(e.to_string())
        })?;

        Ok(search_response)
    }
}

#[derive(Debug, Serialize)]
struct SearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_hits: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    #[serde(default)]
    pub hits: Vec<serde_json::Value>,
    #[serde(default)]
    pub num_hits: u64,
    #[serde(default, rename = "elapsed_time_micros")]
    pub elapsed_time_micros: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum QuickwitError {
    #[error("Quickwit is disabled")]
    Disabled,
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("API error {0}: {1}")]
    ApiError(u16, String),
    #[error("Parse error: {0}")]
    ParseError(String),
}


