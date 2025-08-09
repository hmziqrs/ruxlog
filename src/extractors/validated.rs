use axum::extract::{
    rejection::JsonRejection, rejection::QueryRejection, FromRequest, FromRequestParts, Query,
    Request,
};
use axum::http::request::Parts;
use axum::Json;

use serde::de::DeserializeOwned;
use std::ops::Deref;
use validator::Validate;

use crate::error::ErrorResponse;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send + Sync,
    S: Send + Sync + 'static,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ErrorResponse;

    async fn from_request(req: Request, state: &S) -> Result<Self, ErrorResponse> {
        match Json::<T>::from_request(req, state).await {
            Ok(json) => {
                let data = json.0;
                match data.validate() {
                    Ok(_) => Ok(ValidatedJson(data)),
                    Err(errors) => {
                        use crate::error::{ErrorCode, ErrorResponse};
                        let errors_json = serde_json::to_value(&errors).unwrap_or_default();
                        Err(ErrorResponse::new(ErrorCode::InvalidInput)
                            .with_message("Validation failed")
                            .with_context(errors_json))
                    }
                }
            }
            Err(err) => Err(err.into()),
        }
    }
}

impl<T> Deref for ValidatedJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ValidatedQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate + Send + Sync,
    S: Send + Sync + 'static,
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>,
{
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ErrorResponse> {
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(query) => {
                let data = query.0;
                match data.validate() {
                    Ok(_) => Ok(ValidatedQuery(data)),
                    Err(errors) => {
                        use crate::error::{ErrorCode, ErrorResponse};
                        let errors_json = serde_json::to_value(&errors).unwrap_or_default();
                        Err(ErrorResponse::new(ErrorCode::InvalidInput)
                            .with_message("Query validation failed")
                            .with_context(errors_json))
                    }
                }
            }
            Err(err) => Err(err.into()),
        }
    }
}

impl<T, S> FromRequest<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate + Send + Sync,
    S: Send + Sync + 'static,
    Query<T>: FromRequest<S, Rejection = QueryRejection>,
{
    type Rejection = ErrorResponse;

    async fn from_request(req: Request, state: &S) -> Result<Self, ErrorResponse> {
        match Query::<T>::from_request(req, state).await {
            Ok(query) => {
                let data = query.0;
                match data.validate() {
                    Ok(_) => Ok(ValidatedQuery(data)),
                    Err(errors) => {
                        use crate::error::{ErrorCode, ErrorResponse};
                        let errors_json = serde_json::to_value(&errors).unwrap_or_default();
                        Err(ErrorResponse::new(ErrorCode::InvalidInput)
                            .with_message("Query validation failed")
                            .with_context(errors_json))
                    }
                }
            }
            Err(err) => Err(err.into()),
        }
    }
}

impl<T> Deref for ValidatedQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
