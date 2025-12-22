use axum::{middleware, routing::post, Router};

use crate::{middlewares::auth_guard, AppState};

pub mod controller;
pub mod validator;

/// Public + authenticated (moderator) and admin comment routes
pub fn routes() -> Router<AppState> {
    // Base routes for creating & managing own comments
    let base = Router::<AppState>::new()
        .route("/create", post(controller::create))
        .route("/update/{comment_id}", post(controller::update))
        .route("/delete/{comment_id}", post(controller::delete))
        .route("/flag/{comment_id}", post(controller::flag))
        .route_layer(middleware::from_fn(auth_guard::verified))
        // Public route for listing comments by post
        .route("/{post_id}", post(controller::find_all_by_post));

    // Admin moderation routes nested under /admin
    let admin = Router::<AppState>::new()
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
        .route_layer(middleware::from_fn(
            auth_guard::verified_with_role::<{ auth_guard::ROLE_ADMIN }>,
        ));

    base.nest("/admin", admin)
}
