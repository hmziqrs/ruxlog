use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_garde::WithValidation;
use axum_macros::debug_handler;

use crate::{modules::auth_v1::validator::V1LoginPayload, AppState};

#[debug_handler]
pub async fn login(
    state: State<AppState>,
    WithValidation(payload): WithValidation<Json<V1LoginPayload>>,
) -> impl IntoResponse {
    print!("{}", state.db_pool.status().max_size);
    println!("login v2 #{:?}", payload);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

// pub async fn register(body: Garde<Json<validator::V1RegisterPayload>>) -> String {
//     user::User::find_all(&mut crate::db::connection::establish_connection()).unwrap();
//     println!("register v2 #{:?}", body);
//     "register v2".to_string()
// }
