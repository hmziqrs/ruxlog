use axum::Json;
// use axum_valid::Valid;
use serde_json::Value;

use  super::validator;

pub async fn login() -> String {
    "login v2".to_string()
}