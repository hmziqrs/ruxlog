use axum::{
    routing::{get, post},
    Router,
};

use super::{controller_v1, controller_v2};

pub fn routes() -> Router {
    let v1 = Router::new().route("/login", post(controller_v1::login));
    let v2 = Router::new().route("/login", post(controller_v2::login));

    let nest = Router::new()
        .nest("/v1", v1)
        .nest("/v2", v2);

    return Router::new().nest("/auth", nest);
}