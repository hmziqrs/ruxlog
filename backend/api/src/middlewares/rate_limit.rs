//! Rate limiting middleware using a Redis-based sliding window counter.
//!
//! Provides per-IP, per-path rate limiting with standard HTTP headers.
//!
//! Usage:
//! ```ignore
//! let router = Router::new()
//!     .route("/login", post(login))
//!     .layer(RateLimitLayer::new(state.clone(), 5, 60));
//! ```

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::response::{IntoResponse, Response};
use tower::{Layer, Service};
use tower_sessions_redis_store::fred::interfaces::LuaInterface;
use tower_sessions_redis_store::fred::types::{FromValue, Value};
use tracing::{debug, warn};

use crate::error::{ErrorCode, ErrorResponse};
use crate::state::AppState;

// Standard rate limit header names (not in http::header module)
static X_RATELIMIT_LIMIT: HeaderName = HeaderName::from_static("x-ratelimit-limit");
static X_RATELIMIT_REMAINING: HeaderName = HeaderName::from_static("x-ratelimit-remaining");
static X_RATELIMIT_RESET: HeaderName = HeaderName::from_static("x-ratelimit-reset");

/// Sliding window counter Lua script.
///
/// KEYS[1] = rate limit key
/// ARGV[1] = max_requests (unused in script, for reference)
/// ARGV[2] = window_secs (TTL)
///
/// Uses INCR + EXPIRE on first request for a fixed-window counter.
/// Returns: [count, ttl]
const SLIDING_WINDOW_SCRIPT: &str = r#"
local key = KEYS[1]
local max_requests = tonumber(ARGV[1])
local ttl = tonumber(ARGV[2])

local count = redis.call('INCR', key)
if count == 1 then
    redis.call('EXPIRE', key, ttl)
end

local current_ttl = redis.call('TTL', key)
if current_ttl < 0 then current_ttl = ttl end

return {count, current_ttl}
"#;

/// Extract the client IP from the request's resolved `ClientIp` extension.
///
/// The extension is populated by the `axum_client_ip` layer (configured via
/// `IP_SOURCE`) at the app root, which centralises the trusted-proxy / header
/// policy. Reading raw `x-forwarded-for` here would let an attacker spoof the
/// rate-limit key. Falls back to `"unknown"` only if the layer was not applied
/// (which itself collapses all such clients into one bucket). See plan 6b.
fn client_ip<B>(request: &axum::http::Request<B>) -> String {
    request
        .extensions()
        .get::<axum_client_ip::ClientIp>()
        .map(|ip| ip.0.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Tower layer that creates a [`RateLimitMiddleware`] for the given configuration.
#[derive(Clone)]
pub struct RateLimitLayer {
    state: AppState,
    max_requests: u64,
    window_secs: u64,
}

impl RateLimitLayer {
    /// Create a new rate limit layer.
    ///
    /// # Arguments
    /// * `state` - Application state (provides access to the Redis pool)
    /// * `max_requests` - Maximum number of requests allowed within the window
    /// * `window_secs` - Duration of the rate limit window in seconds
    pub fn new(state: AppState, max_requests: u64, window_secs: u64) -> Self {
        Self {
            state,
            max_requests,
            window_secs,
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            state: self.state.clone(),
            max_requests: self.max_requests,
            window_secs: self.window_secs,
        }
    }
}

/// Tower middleware service that enforces rate limits via Redis.
#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    state: AppState,
    max_requests: u64,
    window_secs: u64,
}

impl<S> Service<Request> for RateLimitMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let state = self.state.clone();
        let mut inner = self.inner.clone();
        let max_requests = self.max_requests;
        let window_secs = self.window_secs;

        Box::pin(async move {
            let ip = client_ip(&req);
            let path = req.uri().path();
            let key = format!("ratelimit:{}:{}", ip, path);

            let keys = vec![key.clone()];
            let args: Vec<Value> = vec![
                Value::from(max_requests as i64),
                Value::from(window_secs as i64),
            ];

            let result: Result<Vec<Value>, _> = state
                .redis_pool
                .eval(SLIDING_WINDOW_SCRIPT, keys, args)
                .await;

            let (count, ttl) = match result {
                Ok(values) => {
                    let count = u64::from_value(values[0].clone()).unwrap_or(1);
                    let ttl = u64::from_value(values[1].clone()).unwrap_or(window_secs);
                    (count, ttl)
                }
                Err(err) => {
                    // Fail closed: if Redis is unavailable we cannot enforce a
                    // per-IP limit, so rejecting (503) is safer than silently
                    // allowing unbounded traffic (the previous fail-open
                    // behaviour). See plan Phase 6b.
                    warn!(
                        error = %err,
                        key = %key,
                        "Redis error during rate limit check, rejecting (fail-closed)"
                    );
                    let response = (
                        axum::http::StatusCode::SERVICE_UNAVAILABLE,
                        axum::Json(serde_json::json!({
                            "error": "rate limit service unavailable",
                            "message": "Could not reach the rate-limit store. Try again shortly."
                        })),
                    )
                        .into_response();
                    return Ok(response);
                }
            };

            // Check if rate limited
            if count > max_requests {
                debug!(
                    ip = %ip,
                    path,
                    count,
                    max_requests,
                    window_secs,
                    "Rate limit exceeded"
                );

                let retry_after = ttl;
                let body = ErrorResponse::new(ErrorCode::RateLimited)
                    .with_message(format!(
                        "Too many requests. Try again in {} seconds.",
                        retry_after
                    ))
                    .with_retry_after(retry_after);

                let mut response = body.into_response();
                let headers = response.headers_mut();
                insert_header(headers, &X_RATELIMIT_LIMIT, max_requests);
                insert_header(headers, &X_RATELIMIT_REMAINING, 0);
                insert_header(headers, &X_RATELIMIT_RESET, ttl);
                insert_header(headers, &axum::http::header::RETRY_AFTER, retry_after);

                return Ok(response);
            }

            // Request is allowed — run the inner service and attach rate limit headers
            let response = inner.call(req).await?;

            let remaining = max_requests.saturating_sub(count);
            let mut response = response;
            let headers = response.headers_mut();
            insert_header(headers, &X_RATELIMIT_LIMIT, max_requests);
            insert_header(headers, &X_RATELIMIT_REMAINING, remaining);
            insert_header(headers, &X_RATELIMIT_RESET, ttl);

            Ok(response)
        })
    }
}

/// Helper to insert a numeric header value, falling back gracefully.
fn insert_header(headers: &mut axum::http::HeaderMap, name: &HeaderName, value: u64) {
    let val =
        HeaderValue::from_str(&value.to_string()).unwrap_or_else(|_| HeaderValue::from_static("0"));
    headers.insert(name, val);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn client_ip_reads_resolved_extension() {
        // The middleware trusts the axum_client_ip layer, not raw headers.
        let mut req = axum::http::Request::builder().body(()).unwrap();
        req.extensions_mut()
            .insert(axum_client_ip::ClientIp(IpAddr::V4(Ipv4Addr::new(
                203, 0, 113, 50,
            ))));
        assert_eq!(client_ip(&req), "203.0.113.50");
    }

    #[test]
    fn client_ip_ignores_spoofed_headers_without_extension() {
        // An attacker-supplied X-Forwarded-For must NOT be read directly; with
        // no resolved extension the limiter falls back to "unknown".
        let req = axum::http::Request::builder()
            .header("x-forwarded-for", "1.2.3.4")
            .header("x-real-ip", "5.6.7.8")
            .body(())
            .unwrap();
        assert_eq!(client_ip(&req), "unknown");
    }

    #[test]
    fn client_ip_fallback_to_unknown() {
        let req = axum::http::Request::builder().body(()).unwrap();
        assert_eq!(client_ip(&req), "unknown");
    }
}
