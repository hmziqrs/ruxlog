use std::env;

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::error::{ErrorCode, ErrorResponse};

pub fn get_static_csrf_key() -> String {
    let key = env::var("CSRF_KEY").unwrap_or_else(|_| "ultra-instinct-goku".to_string());
    return key;
}

pub async fn csrf_guard(req: Request, next: Next) -> Result<Response, Response> {
    let err_resp = ErrorResponse::new(ErrorCode::InvalidToken)
        .with_message("Request is from an unverified client");
    if let Some(token) = req.headers().get("csrf-token") {
        if let Ok(token_str) = token.to_str() {
            use base64::prelude::*;

            let parsed_token = BASE64_STANDARD.decode(token_str);
            match parsed_token {
                Ok(parsed_token) => {
                    let parsed_token = String::from_utf8(parsed_token);
                    if parsed_token.is_err() {
                        return Err(err_resp.clone().into_response());
                    }
                    if parsed_token.unwrap() != get_static_csrf_key() {
                        return Err(err_resp.clone().into_response());
                    }

                    Ok(next.run(req).await)
                }
                Err(_) => {
                    return Err(err_resp.clone().into_response());
                }
            }
        } else {
            Err(err_resp.clone().into_response())
        }
    } else {
        Err(err_resp.into_response())
    }
}

pub async fn test(req: Request, next: Next) -> Response {
    next.run(req).await
}
