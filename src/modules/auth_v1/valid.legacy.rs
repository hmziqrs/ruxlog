use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
    response::{IntoResponse, Response},
    Json,
};
use axum_valid::{Valid, ValidRejection};
use serde_json::json;
use validator::ValidationErrors;

pub struct JsonValid<T>(pub T);

impl<T> std::ops::Deref for JsonValid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> JsonValid<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for JsonValid<Valid<T>>
where
    S: Send + Sync,
    Valid<T>: FromRequest<S>,
    T: Send,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Valid::<T>::from_request(req, state).await {
            Ok(valid) => Ok(JsonValid(valid)),
            Err(err) => Err(format_error(ValidRejection::Inner(err)).into_response()),
        }
    }
}

#[async_trait]
impl<S, T> FromRequestParts<S> for JsonValid<Valid<T>>
where
    S: Send + Sync,
    Valid<T>: FromRequestParts<S>,
    T: Send,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Valid::<T>::from_request_parts(parts, state).await {
            Ok(valid) => Ok(JsonValid(valid)),
            Err(err) => Err(format_error(ValidRejection::Inner(err)).into_response()),
        }
    }
}

fn format_error<E>(err: ValidRejection<E>) -> Json<serde_json::Value> {
    match err {
        ValidRejection::Valid(errors) => Json(json!({
            "message": "Validation failed",
            "errors": format_validation_errors(errors)
        })),
        ValidRejection::Inner(e) => {
            println!("test error: ");
            Json(json!({
                "message": "An error occurred while processing the request",
            }))
        }
    }
}

fn format_validation_errors(errors: ValidationErrors) -> serde_json::Value {
    let mut formatted_errors = serde_json::Map::new();
    for (field, error_vec) in errors.field_errors() {
        let messages: Vec<String> = error_vec
            .iter()
            .map(|error| {
                error
                    .message
                    .as_ref()
                    .map(|cow| cow.to_string())
                    .unwrap_or_else(|| "Invalid input".to_string())
            })
            .collect();
        formatted_errors.insert(field.to_string(), json!(messages));
    }
    json!(formatted_errors)
}
