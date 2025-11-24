pub mod controller;
pub mod validator;

use axum::{extract::DefaultBodyLimit, middleware, routing::post, Router};
use axum_login::login_required;

use crate::{
    config,
    middlewares::{user_permission, user_status},
    services::auth::AuthBackend,
    AppState,
};

pub fn routes() -> Router<AppState> {
    let post_limited = Router::new()
        .route("/create", post(controller::create))
        .route("/update/{post_id}", post(controller::update))
        .route("/autosave", post(controller::autosave))
        .layer(DefaultBodyLimit::max(config::body_limits::POST));

    let protected = Router::new()
        .route("/query", post(controller::query))
        .route("/delete/{post_id}", post(controller::delete))
        .route(
            "/revisions/{post_id}/list",
            post(controller::revisions_list),
        )
        .route(
            "/revisions/{post_id}/restore/{revision_id}",
            post(controller::revisions_restore),
        )
        .route("/schedule", post(controller::schedule))
        .route("/series/create", post(controller::series_create))
        .route(
            "/series/update/{series_id}",
            post(controller::series_update),
        )
        .route(
            "/series/delete/{series_id}",
            post(controller::series_delete),
        )
        .route("/series/list", post(controller::series_list))
        .route(
            "/series/add/{post_id}/{series_id}",
            post(controller::series_add),
        )
        .route(
            "/series/remove/{post_id}/{series_id}",
            post(controller::series_remove),
        )
        .merge(post_limited)
        .route_layer(middleware::from_fn(user_permission::author))
        .route_layer(middleware::from_fn(user_status::only_verified))
        .route_layer(login_required!(AuthBackend));

    // Routes requiring authentication (any logged-in user)
    let authenticated = Router::new()
        .route("/like/{post_id}", post(controller::like_post))
        .route("/unlike/{post_id}", post(controller::unlike_post))
        .route("/like/status/{post_id}", post(controller::like_status))
        .route("/like/status/batch", post(controller::like_status_batch))
        .route_layer(login_required!(AuthBackend));

    let public = Router::new()
        .route("/view/{id_or_slug}", post(controller::find_by_id_or_slug))
        .route("/list/published", post(controller::find_published_posts))
        .route("/sitemap", post(controller::sitemap))
        .route("/track_view/{post_id}", post(controller::track_view));

    protected.merge(authenticated).merge(public)
}
