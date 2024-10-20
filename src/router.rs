use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Json},
    routing::{self, get, post, put},
    Router,
};
use axum_login::{login_required, permission_required};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::{
    middlewares::{user_permission, user_status},
    modules::{category_v1, post_v1, tag_v1},
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

    // let csrf_v1_routes: Router<AppState> =
    //     Router::new()
    //     .route("/check", post(csrf_v1::controller::check_key))
    //     .route_layer(login_required!(AuthBackend))
    //     .route("/get", post(csrf_v1::controller::get_key));

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
        .route("/create", post(post_v1::controller::create))
        .route("/update/:post_id", post(post_v1::controller::update))
        .route("/delete/:post_id", post(post_v1::controller::delete))
        .route(
            "/view/:id_or_slug",
            post(post_v1::controller::find_by_id_or_slug),
        )
        .route(
            "/view/published",
            post(post_v1::controller::find_published_posts),
        )
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    let post_comment_v1_routes = Router::new()
        .route("/create", post(post_comment_v1::controller::create))
        .route(
            "/update/:comment_id",
            post(post_comment_v1::controller::update),
        )
        .route(
            "/delete/:comment_id",
            post(post_comment_v1::controller::delete),
        )
        .route("/list", get(post_comment_v1::controller::list_all))
        .route(
            "/list/paginated",
            get(post_comment_v1::controller::list_paginated),
        )
        .route(
            "/list/query",
            get(post_comment_v1::controller::list_with_query),
        )
        .route(
            "/list/post/:post_id",
            get(post_comment_v1::controller::list_by_post),
        )
        .route(
            "/list/user/:user_id",
            get(post_comment_v1::controller::list_by_user),
        )
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    let category_v1_routes = Router::new()
        .route("/create", post(category_v1::controller::create))
        .route(
            "/view/:category_id",
            get(category_v1::controller::get_category_by_id),
        )
        .route("/list", get(category_v1::controller::get_categories))
        .route(
            "/update/:category_id",
            post(category_v1::controller::update),
        )
        .route(
            "/delete/:category_id",
            post(category_v1::controller::delete),
        )
        .route_layer(middleware::from_fn(user_permission::admin))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));
    // .route_layer(permission_required!(AuthBackend, "user"));

    let tag_v1_routes = Router::new()
        .route("/create", post(tag_v1::controller::create))
        .route("/update/:tag_id", post(tag_v1::controller::update))
        .route("/delete/:tag_id", post(tag_v1::controller::delete))
        .route("/view/:tag_id", get(tag_v1::controller::find_by_id))
        .route("/list", get(tag_v1::controller::find_all))
        .route("/list/query", get(tag_v1::controller::find_with_query))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    Router::new()
        .route("/", routing::get(handler))
        .nest("/auth/v1", auth_v1_routes)
        .nest("/user/v1", user_v1_routes)
        .nest("/email_verification/v1", email_verification_v1_routes)
        .nest("/forgot_password/v1", forgot_password_v1_routes)
        .nest("/post/v1", post_v1_routes)
        .nest("/post/comment/v1", post_comment_v1_routes)
        .nest("/category/v1", category_v1_routes)
        .nest("/tag/v1", tag_v1_routes)
        .layer(TraceLayer::new_for_http())
}

async fn handler(s: State<AppState>) -> impl IntoResponse {
    println!("{:?}", s.db_pool.status());
    (StatusCode::OK, Json(json!({"message": "success"})))
}
