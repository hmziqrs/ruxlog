use axum::Json;
use axum_valid::Garde;

use super::validator;

pub async fn login(body: Garde<Json<validator::V1LoginPayload>>) -> String {
    println!("login v2 #{:?}", body);
    "login v2".to_string()
}
