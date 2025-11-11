use super::config::{get_base_url, get_csrf_token};
use super::FormData;
use reqwest::{Client, RequestBuilder as ReqwestRequestBuilder};
use serde::{de::DeserializeOwned, Serialize};

/// Unified error type for HTTP operations
#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Decode(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(e) => write!(f, "{}", e),
            Error::Decode(e) => write!(f, "Decode error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

/// Request wrapper for reqwest
pub struct Request {
    pub(crate) inner: ReqwestRequestBuilder,
}

pub type RequestBuilder = Request;

impl Request {
    pub async fn send(self) -> Result<Response, Error> {
        let resp = self.inner.send().await?;
        Response::from_reqwest(resp).await
    }
}

/// Unified response wrapper that provides a clean API for both platforms
pub struct Response {
    status: u16,
    body: Vec<u8>,
}

impl Response {
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Get response body as text without consuming self
    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    pub async fn text(self) -> Result<String, Error> {
        String::from_utf8(self.body)
            .map_err(|e| Error::Decode(format!("UTF-8 decode error: {}", e)))
    }

    pub async fn json<T: DeserializeOwned>(self) -> Result<T, Error> {
        serde_json::from_slice(&self.body).map_err(|e| Error::Decode(e.to_string()))
    }
}

impl Response {
    pub async fn from_reqwest(resp: reqwest::Response) -> Result<Self, Error> {
        let status = resp.status().as_u16();
        let body = resp.bytes().await?.to_vec();
        Ok(Response { status, body })
    }
}

// ============================================================================
// HTTP Helper Functions
// ============================================================================

thread_local! {
    static CLIENT: Client = Client::new();
}

fn create_headers(req: ReqwestRequestBuilder) -> ReqwestRequestBuilder {
    req.header("Content-Type", "application/json")
        .header("csrf-token", get_csrf_token())
}

pub fn get(endpoint: &str) -> Request {
    let url = format!("{}{}", get_base_url(), endpoint);
    CLIENT.with(|client| {
        let req = client.get(&url);
        Request {
            inner: create_headers(req),
        }
    })
}

pub fn post<T: Serialize>(endpoint: &str, body: &T) -> Request {
    let url = format!("{}{}", get_base_url(), endpoint);
    CLIENT.with(|client| {
        let req = client.post(&url);
        Request {
            inner: create_headers(req).json(body),
        }
    })
}

pub fn put<T: Serialize>(endpoint: &str, body: &T) -> Request {
    let url = format!("{}{}", get_base_url(), endpoint);
    CLIENT.with(|client| {
        let req = client.put(&url);
        Request {
            inner: create_headers(req).json(body),
        }
    })
}

pub fn delete(endpoint: &str) -> Request {
    let url = format!("{}{}", get_base_url(), endpoint);
    CLIENT.with(|client| {
        let req = client.delete(&url);
        Request {
            inner: create_headers(req),
        }
    })
}

pub fn post_multipart(endpoint: &str, form_data: &FormData) -> Result<Request, String> {
    let url = format!("{}{}", get_base_url(), endpoint);

    CLIENT.with(|client| {
        let mut form = reqwest::multipart::Form::new();

        if let Some(obj) = form_data.as_object() {
            for (key, value) in obj {
                match value {
                    serde_json::Value::String(s) => {
                        form = form.text(key.clone(), s.clone());
                    }
                    serde_json::Value::Number(n) => {
                        form = form.text(key.clone(), n.to_string());
                    }
                    serde_json::Value::Bool(b) => {
                        form = form.text(key.clone(), b.to_string());
                    }
                    serde_json::Value::Null => {
                        form = form.text(key.clone(), "");
                    }
                    _ => {
                        return Err(format!("Unsupported value type for key '{}'", key));
                    }
                }
            }
        }

        let req = client.post(&url);
        Ok(Request {
            inner: req.header("csrf-token", get_csrf_token()).multipart(form),
        })
    })
}
