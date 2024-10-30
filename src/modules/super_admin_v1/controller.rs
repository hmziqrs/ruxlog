use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use serde_json::json;

use crate::{
    db::models::user::User,
    services::auth::{AuthSession, Credentials},
    AppState,
};
#[debug_handler]
pub async fn pool_stats(state: State<AppState>) -> impl IntoResponse {
    let pool = &state.db_pool;
    // println!("pool: {:?}", pool);
    println!("pool is_closed: {:?}", pool.is_closed());
    println!("pool.sattus: {:?}", pool.status());
    println!("pool.manager: {:?}", pool.manager());
    println!("pool.timeouts: {:?}", pool.timeouts());
    match pool.get().await {
        Ok(conn) => {
            println!("pool get: {:?}", true);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    (StatusCode::OK, Json(json!({"message": "test" })))
}

pub async fn close(state: State<AppState>) -> impl IntoResponse {
    let pool = &state.db_pool;
    println!("pre close : {:?}", pool.is_closed());
    pool.close();
    println!("pool is_closed: {:?}", pool.is_closed());
    (StatusCode::OK, Json(json!({"message": "disconnected" })))
}
