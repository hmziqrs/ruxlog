use axum::Json;
use axum_valid::Garde;

use crate::db::models::user;

use super::validator;


pub async fn login(body: Garde<Json<validator::V1LoginPayload>>) -> String {
    println!("login v2 #{:?}", body);
    "login v2".to_string()
}

// pub async fn register(body: Garde<Json<validator::V1RegisterPayload>>) -> String {
//     user::User::find_all(&mut crate::db::connection::establish_connection()).unwrap();
//     println!("register v2 #{:?}", body);
//     "register v2".to_string()
// }
