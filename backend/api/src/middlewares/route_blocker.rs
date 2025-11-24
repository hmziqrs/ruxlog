use crate::error::RouteBlockerError;
use crate::services::route_blocker_service::RouteBlockerService;
use crate::state::AppState;
use axum::{
    extract::{MatchedPath, Request, State},
    response::{IntoResponse, Response},
};
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct RouteBlockerLayer {
    state: AppState,
}

impl RouteBlockerLayer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl<S> Layer<S> for RouteBlockerLayer {
    type Service = RouteBlockerMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RouteBlockerMiddleware {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RouteBlockerMiddleware<S> {
    inner: S,
    state: AppState,
}

impl<S> Service<Request> for RouteBlockerMiddleware<S>
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

        Box::pin(async move {
            let path = req.uri().path().to_string();
            let matched_pattern = req
                .extensions()
                .get::<MatchedPath>()
                .map(|matched| matched.as_str().to_string());
            let pattern = matched_pattern.clone().unwrap_or_else(|| path.clone());

            let is_development = env::var("APP_ENV")
                .unwrap_or_else(|_| "development".to_string())
                == "development";

            if is_development {
                debug!(path, "Route blocker disabled in development mode");
                return inner.call(req).await;
            }

            info!("ROUTER BLOCKER WORKING");

            if let Some(_) = matched_pattern {
                if let Err(err) = RouteBlockerService::record_route_pattern(&state, &pattern).await
                {
                    error!(
                        pattern = %pattern,
                        error = %err,
                        "Failed to record route pattern in cache"
                    );
                }
            }

            match RouteBlockerService::is_route_blocked(State(state.clone()), &pattern).await {
                Ok(true) => {
                    warn!(path = %path, pattern = %pattern, "Route blocked by dynamic route_blocker middleware");
                    let error_response: Response = RouteBlockerError::Blocked { path }.into_response();
                    return Ok(error_response);
                }
                Ok(false) => {
                    debug!(path = %path, pattern = %pattern, "Route allowed");
                }
                Err(e) => {
                    error!(error = %e, path = %path, pattern = %pattern, "Failed to check route status");
                    let error_response: Response = RouteBlockerError::CheckFailed(e.to_string()).into_response();
                    return Ok(error_response);
                }
            }

            inner.call(req).await
        })
    }
}
