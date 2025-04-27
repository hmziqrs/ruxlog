use async_trait::async_trait;
use axum::extract::{rejection::JsonRejection, FromRequest, Request};
use axum::Json;
use axum_macros::debug_handler;
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
