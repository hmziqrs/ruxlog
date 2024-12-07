use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::middlewares::static_csrf::get_static_csrf_key;

// use axum_csrf::CsrfToken;
// use serde::{Deserialize, Serialize};

pub async fn generate() -> impl IntoResponse {
    use base64::prelude::*;
    let static_csrf = get_static_csrf_key();
    let token = BASE64_STANDARD.encode(static_csrf);

    (
        StatusCode::OK,
        Json(json!({"message": "csrf token generated successfully", "token": token})),
    )
}

// #[derive(Deserialize, Serialize)]
// pub struct Keys {
//     authenticity_token: String,
//     // Your attributes...
// }

// pub async fn get_key(token: CsrfToken) -> impl IntoResponse {
//     let keys = Keys {
//         authenticity_token: token.authenticity_token().unwrap(),
//     };

//     // We must return the token so that into_response will run and add it to our response cookies.
//     (token, axum::Json(keys))
// }

// pub async fn check_key() -> &'static str {
//     "Token is Valid lets do stuff!"
// }
