use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use axum_login::login_required;
use tower_http::trace::TraceLayer;

use crate::{
    middlewares::{route_blocker::block_routes, user_permission, user_status},
    modules::{asset_v1, category_v1, feed_v1, post_v1, seed_v1, tag_v1},
};
use crate::{modules::post_comment_v1, services::auth::AuthBackend};

use super::{
    modules::{auth_v1, email_verification_v1, forgot_password_v1, user_v1},
    AppState,
};

pub fn router() -> Router<AppState> {
    let auth_v1_routes = Router::new()
        .route("/register", post(auth_v1::controller::register))
        .route("/log_in", post(auth_v1::controller::log_in))
        .route_layer(middleware::from_fn(user_status::only_unauthenticated))
        .merge(
            Router::new()
                .route("/log_out", post(auth_v1::controller::log_out))
                .route_layer(middleware::from_fn(user_status::only_authenticated)),
        );

    let user_v1_routes = Router::new()
        .route("/update", put(user_v1::controller::update_profile))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route("/get", get(user_v1::controller::get_profile))
        .route_layer(login_required!(AuthBackend));

    let email_verification_v1_routes = Router::new()
        .route("/verify", post(email_verification_v1::controller::verify))
        .route("/resend", post(email_verification_v1::controller::resend))
        .route_layer(middleware::from_fn(user_status::only_unverified))
        .route_layer(login_required!(AuthBackend));

    let forgot_password_v1_routes = Router::new()
        .route("/request", post(forgot_password_v1::controller::generate))
        .route("/verify", post(forgot_password_v1::controller::verify))
        .route("/reset", post(forgot_password_v1::controller::reset))
        .route_layer(middleware::from_fn(user_status::only_unauthenticated));

    let post_v1_routes = Router::new()
        .route("/query", post(post_v1::controller::query))
        .route_layer(middleware::from_fn(user_permission::author))
        .route("/create", post(post_v1::controller::create))
        .route("/update/{post_id}", post(post_v1::controller::update))
        .route("/delete/{post_id}", post(post_v1::controller::delete))
        .route_layer(middleware::from_fn(user_permission::author))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
        .route(
            "/view/{id_or_slug}",
            post(post_v1::controller::find_by_id_or_slug),
        )
        .route(
            "/list/published",
            post(post_v1::controller::find_published_posts),
        )
        .route("/sitemap", post(post_v1::controller::sitemap))
        .route(
            "/track_view/{post_id}",
            post(post_v1::controller::track_view),
        );

    let post_comment_v1_routes = Router::new()
        .route("/list", get(post_comment_v1::controller::list))
        .route_layer(middleware::from_fn(user_permission::moderator))
        .route("/create", post(post_comment_v1::controller::create))
        .route(
            "/update/{comment_id}",
            post(post_comment_v1::controller::update),
        )
        .route(
            "/delete/{comment_id}",
            post(post_comment_v1::controller::delete),
        )
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
        .route(
            "/list/{post_id}",
            get(post_comment_v1::controller::list_by_post),
        );

    let category_v1_routes = Router::new()
        .route("/create", post(category_v1::controller::create))
        .route(
            "/update/{category_id}",
            post(category_v1::controller::update),
        )
        .route(
            "/delete/{category_id}",
            post(category_v1::controller::delete),
        )
        .route(
            "/list/query",
            post(category_v1::controller::find_with_query),
        )
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
        .route("/list", get(category_v1::controller::find_all))
        .route(
            "/view/{category_id}",
            get(category_v1::controller::find_by_id_or_slug),
        );

    let tag_v1_routes = Router::new()
        .route("/create", post(tag_v1::controller::create))
        .route("/update/{tag_id}", post(tag_v1::controller::update))
        .route("/delete/{tag_id}", post(tag_v1::controller::delete))
        .route("/view/{tag_id}", post(tag_v1::controller::find_by_id))
        .route("/list/query", post(tag_v1::controller::find_with_query))
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend))
        .route("/list", get(tag_v1::controller::find_all));

    let admin_user_v1_routes = Router::new()
        .route("/list", post(user_v1::controller::admin_list))
        .route("/view/{user_id}", get(user_v1::controller::admin_view))
        .route("/create", post(user_v1::controller::admin_create))
        .route("/update/{user_id}", post(user_v1::controller::admin_update))
        .route("/delete/{user_id}", post(user_v1::controller::admin_delete))
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    let seed_routes = Router::new()
        .route("/seed_tags", post(seed_v1::controller::seed_tags))
        .route(
            "/seed_categories",
            post(seed_v1::controller::seed_categories),
        )
        .route("/seed_posts", post(seed_v1::controller::seed_posts))
        .route(
            "/seed_post_comments",
            post(seed_v1::controller::seed_post_comments),
        )
        .route("/seed", post(seed_v1::controller::seed));

    Router::new()
        .layer(middleware::from_fn(block_routes))
        .nest("/auth/v1", auth_v1_routes)
        .nest("/user/v1", user_v1_routes)
        .nest("/email_verification/v1", email_verification_v1_routes)
        .nest("/forgot_password/v1", forgot_password_v1_routes)
        .nest("/post/v1", post_v1_routes)
        .nest("/post/comment/v1", post_comment_v1_routes)
        .nest("/category/v1", category_v1_routes)
        .nest("/tag/v1", tag_v1_routes)
        .nest("/admin/user/v1", admin_user_v1_routes)
        .nest("/asset/v1", asset_v1::routes())
        .nest("/feed/v1", feed_v1::routes())
        .nest("/admin/seed/v1", seed_routes)
        .layer(TraceLayer::new_for_http())
}
