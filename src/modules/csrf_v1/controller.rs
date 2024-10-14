// pub async fn get() -> impl Responder {
//     // HttpResponse::Ok().body("Hello world!")
// }

use axum::response::IntoResponse;
use axum_csrf::CsrfToken;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Keys {
    authenticity_token: String,
    // Your attributes...
}

pub async fn get_key(token: CsrfToken) -> impl IntoResponse {
    let keys = Keys {
        authenticity_token: token.authenticity_token().unwrap(),
    };

    // We must return the token so that into_response will run and add it to our response cookies.
    (token, axum::Json(keys))
}

pub async fn check_key() -> &'static str {
    "Token is Valid lets do stuff!"
}
