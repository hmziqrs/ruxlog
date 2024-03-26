use axum::Router;


mod auth;

pub fn routes() -> Router {
    Router::new().merge(auth::router::routes())
}