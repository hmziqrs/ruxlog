use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::error;

#[derive(Clone, Debug)]
pub struct OpenObserveConfig {
    pub endpoint: String,
    pub organization: String,
    pub auth_header: String,
    pub enabled: bool,
}

impl OpenObserveConfig {
    pub fn from_env() -> Self {
        let endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
        let auth_headers = env::var("OTEL_EXPORTER_OTLP_HEADERS").ok();

        let enabled = endpoint.is_some();

        let endpoint = endpoint.unwrap_or_else(|| "http://localhost:5080/api/default".to_string());
        let organization = Self::extract_org(&endpoint);
        let auth_header = auth_headers
            .unwrap_or_else(|| "Basic cm9vdEBleGFtcGxlLmNvbTpDb21wbGV4cGFzcyMxMjM=".to_string());

        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            organization,
            auth_header,
            enabled,
        }
    }

    fn extract_org(endpoint: &str) -> String {
        endpoint
            .split("/api/")
            .nth(1)
            .unwrap_or("default")
            .split('/')
            .next()
            .unwrap_or("default")
            .to_string()
    }
}

#[derive(Clone)]
pub struct OpenObserveClient {
    client: Client,
    config: OpenObserveConfig,
}

impl OpenObserveClient {
    pub fn new(config: OpenObserveConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn search(
        &self,
        stream: &str,
        sql: &str,
        start_time: i64,
        end_time: i64,
        from: i64,
        size: i64,
    ) -> Result<SearchResponse, OpenObserveError> {
        if !self.config.enabled {
            return Err(OpenObserveError::Disabled);
        }

        let url = format!(
            "{}/{}/_search",
            self.config.endpoint, self.config.organization
        );

        let sql_with_stream = sql.replace("{stream}", stream);

        let request = SearchRequest {
            query: QueryParams {
                sql: sql_with_stream,
                start_time,
                end_time,
                from,
                size,
            },
            search_type: Some("ui".to_string()),
            timeout: Some(60),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.config.auth_header)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send request to OpenObserve");
                OpenObserveError::RequestFailed(e.to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "OpenObserve API error");
            return Err(OpenObserveError::ApiError(status.as_u16(), body));
        }

        let search_response = response.json::<SearchResponse>().await.map_err(|e| {
            error!(error = %e, "Failed to parse OpenObserve response");
            OpenObserveError::ParseError(e.to_string())
        })?;

        Ok(search_response)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: QueryParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
    pub sql: String,
    pub start_time: i64,
    pub end_time: i64,
    pub from: i64,
    pub size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub took: i64,
    pub hits: Vec<serde_json::Value>,
    pub total: i64,
    pub from: i64,
    pub size: i64,
    pub scan_size: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum OpenObserveError {
    #[error("OpenObserve is disabled")]
    Disabled,
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("API error {0}: {1}")]
    ApiError(u16, String),
    #[error("Parse error: {0}")]
    ParseError(String),
}
