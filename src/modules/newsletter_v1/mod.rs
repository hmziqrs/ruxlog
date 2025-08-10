use axum::{middleware, routing::post, Router};
use axum_login::login_required;

use crate::{
    middlewares::{user_permission, user_status},
    services::auth::AuthBackend,
    AppState,
};

pub mod controller;
pub mod validator;

pub fn routes() -> Router<AppState> {
    let public = Router::new()
        .route("/subscribe", post(controller::subscribe))
        .route("/unsubscribe", post(controller::unsubscribe));

    let admin = Router::new()
        .route("/send", post(controller::send))
        .route("/subscribers/list", post(controller::list_subscribers))
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    public.merge(admin)
}
