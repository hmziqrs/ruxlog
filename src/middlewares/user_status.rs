use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::error::{ErrorCode, ErrorResponse};
use crate::services::auth::AuthSession;

pub async fn only_verified(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    if let Some(user) = auth.user {
        if !user.is_verified {
            return Err(ErrorResponse::new(ErrorCode::EmailVerificationRequired)
                .with_message("User not verified")
                .into_response());
        }
    }
    Ok(next.run(request).await)
}

pub async fn only_unverified(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    if let Some(user) = auth.user {
        if user.is_verified {
            return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
                .with_message("Resource not available")
                .into_response());
        }
    }
    Ok(next.run(request).await)
}

pub async fn only_unauthenticated(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    if !auth.user.is_none() {
        return Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
            .with_message("Resource not available")
            .into_response());
    }
    Ok(next.run(request).await)
}

pub async fn only_authenticated(
    auth: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    if auth.user.is_none() {
        return Err(ErrorResponse::new(ErrorCode::Unauthorized)
            .with_message("Resource not available")
            .into_response());
    }
    Ok(next.run(request).await)
}

//     required_permission: String,
// ) -> impl Fn(AuthSession, Request, Next) -> impl Future<Output = Result<Response, Response>> {
//     move |auth: AuthSession, request: Request, next: Next| async move {
//                 Ok(next.run(request).await)
//             } else {
//                 Ok((
//                 )
//             }
//         } else {
//             Ok((
//             )
//         }
//     }
// }
