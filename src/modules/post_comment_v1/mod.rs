use axum::{middleware, routing::post, Router};
use axum_login::login_required;

use crate::{
    middlewares::{user_permission, user_status},
    services::auth::AuthBackend,
    AppState,
};

pub mod controller;
pub mod validator;

/// Public + authenticated (moderator) and admin comment routes
pub fn routes() -> Router<AppState> {
    // Base (moderator + verified + login) routes for creating & managing own comments plus listing
    let base = Router::new()
        .route("/list", post(controller::list))
        .route_layer(middleware::from_fn(user_permission::moderator))
        .route("/create", post(controller::create))
        .route("/update/{comment_id}", post(controller::update))
        .route("/delete/{comment_id}", post(controller::delete))
        .route("/flag/{comment_id}", post(controller::flag))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
        .route("/list/{post_id}", post(controller::list_by_post));

    // Admin moderation routes
    let admin = Router::new()
        .route("/list", post(controller::admin_list))
        .route("/flagged", post(controller::admin_flagged))
        .route("/hide/{comment_id}", post(controller::admin_hide))
        .route("/unhide/{comment_id}", post(controller::admin_unhide))
        .route("/delete/{comment_id}", post(controller::admin_delete))
        .route(
            "/flags/clear/{comment_id}",
            post(controller::admin_flags_clear),
        )
        .route("/flags/list", post(controller::admin_flags_list))
        .route(
            "/flags/summary/{comment_id}",
            post(controller::admin_flags_summary),
        )
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    base.merge(admin)
}
