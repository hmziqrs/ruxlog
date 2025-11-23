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
    // Base routes for creating & managing own comments
    let base = Router::new()
        .route("/create", post(controller::create))
        .route("/update/{comment_id}", post(controller::update))
        .route("/delete/{comment_id}", post(controller::delete))
        .route("/flag/{comment_id}", post(controller::flag))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
        // Public route for listing comments by post
        .route("/{post_id}", post(controller::find_all_by_post));

    // Admin moderation routes nested under /admin
    let admin = Router::new()
        .route("/list", post(controller::find_with_query))
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
        .route(
            "/flags/details/{comment_id}",
            post(controller::admin_flags_details),
        )
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    base.nest("/admin", admin)
}
