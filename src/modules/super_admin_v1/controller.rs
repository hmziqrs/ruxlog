use axum::{ http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use serde_json::json;

#[debug_handler]
pub async fn pool_stats() -> impl IntoResponse {
    // let pool = &state.db_pool;
    // // println!("pool: {:?}", pool);
    // println!("pool is_closed: {:?}", pool.is_closed());
    // println!("pool.status: {:?}", pool.status());
    // println!("pool.manager: {:?}", pool.manager());
    // println!("pool.timeouts: {:?}", pool.timeouts());
    // match pool.get().await {
    //     Ok(_) => {
    //         println!("pool get: {:?}", true);
    //     }
    //     Err(e) => {
    //         println!("Error: {:?}", e);
    //     }
    // }

    (StatusCode::OK, Json(json!({"message": "test" })))
}

pub async fn close() -> impl IntoResponse {
    // let pool = &state.db_pool;
    // println!("pre close : {:?}", pool.is_closed());
    // pool.close();
    // println!("pool is_closed: {:?}", pool.is_closed());
    (StatusCode::OK, Json(json!({"message": "disconnected" })))
}
