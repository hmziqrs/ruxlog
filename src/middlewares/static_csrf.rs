use std::convert::Infallible;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

// pub async fn csrf_gaurd(req: Request, next: Next) -> Result<Response, impl IntoResponse> {
pub async fn csrf_gaurd(req: Request, next: Next) -> Result<Response, Response> {
    let err_json = Json(
        json!({"error": "invalid request", "message": "requset is from a un verified client" }),
    );
    if let Some(token) = req.headers().get("csrf-token") {
        if let Ok(token_str) = token.to_str() {
            println!("{}", token_str);
            Ok(next.run(req).await)
        } else {
            Err((StatusCode::BAD_REQUEST, err_json).into_response())
        }
    } else {
        Err((StatusCode::FORBIDDEN, err_json).into_response())
    }
}

pub async fn test(req: Request, next: Next) -> Response {
    println!("test middleware");
    println!("test middleware");
    println!("test middleware");
    next.run(req).await
}
