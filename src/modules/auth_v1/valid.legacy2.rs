use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, FromRequestParts, Request},
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
impl<S, T> FromRequest<S> for JsonValid<Valid<Json<T>>>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned + validator::Validate,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(json) => match json.validate() {
                Ok(_) => Ok(JsonValid(Valid(json))),
                Err(validation_errors) => {
                    Err(format_error(ValidRejection::Valid(validation_errors)).into_response())
                }
            },
            Err(err) => Err(format_json_error(err).into_response()),
        }
    }
}

#[async_trait]
impl<S, T> FromRequestParts<S> for JsonValid<Valid<Json<T>>>
where
    S: Send + Sync,
    T: serde::de::DeserializeOwned + validator::Validate,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // This implementation might not be needed for Json<T>, but included for completeness
        Err(Json(
            json!({"message": "JsonValid<Valid<Json<T>>> cannot be extracted from request parts"}),
        )
        .into_response())
    }
}

fn format_error(err: ValidRejection<ValidationErrors>) -> Json<serde_json::Value> {
    match err {
        ValidRejection::Valid(errors) => Json(json!({
            "message": "Validation failed",
            "errors": format_validation_errors(errors)
        })),
        ValidRejection::Inner(_) => Json(json!({
            "message": "An unexpected error occurred during validation",
        })),
    }
}

fn format_json_error(err: JsonRejection) -> Json<serde_json::Value> {
    Json(json!({
        "message": "Failed to parse JSON",
        "error": err.to_string()
    }))
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
