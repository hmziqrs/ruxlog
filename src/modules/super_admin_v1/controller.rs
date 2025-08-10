use axum::{ http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use serde_json::json;

#[debug_handler]
pub async fn pool_stats() -> impl IntoResponse {
    //     Ok(_) => {
    //     }
    //     Err(e) => {
    //     }
    // }

    (StatusCode::OK, Json(json!({"message": "test" })))
}

pub async fn close() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"message": "disconnected" })))
}
