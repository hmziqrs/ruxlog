use axum::{middleware, routing::post, Router};

use crate::{middlewares::auth_guard, AppState};

pub mod controller;
pub mod validator;

pub fn routes() -> Router<AppState> {
    let public = Router::<AppState>::new()
        .route("/subscribe", post(controller::subscribe))
        .route("/unsubscribe", post(controller::unsubscribe))
        .route("/confirm", post(controller::confirm));

    let admin = Router::<AppState>::new()
        .route("/send", post(controller::send))
        .route("/subscribers/list", post(controller::list_subscribers))
        .route_layer(middleware::from_fn(auth_guard::verified_with_role::<{ auth_guard::ROLE_ADMIN }>));

    public.merge(admin)
}
