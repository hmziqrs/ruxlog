use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use lettre::{message::header::ContentType, AsyncTransport, Message};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::sea_models::newsletter_subscriber::{
        Column as SubscriberColumn, Entity as SubscriberEntity, NewSubscriber, SubscriberQuery,
        SubscriberStatus,
    },
    error::{ErrorCode, ErrorResponse},
    extractors::ValidatedJson,
    services::{
        abuse_limiter::{limiter, AbuseLimiterConfig},
        auth::AuthSession,
    },
    AppState,
};

use super::validator::{
    V1ListSubscribersQuery, V1SendNewsletterPayload, V1SubscribePayload, V1UnsubscribePayload,
};

fn generic_internal_error() -> ErrorResponse {
    ErrorResponse::new(ErrorCode::InternalServerError).with_message("Internal server error")
}

async fn send_mail(
    mailer: &lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
    to_email: &str,
    subject: &str,
    html: Option<&str>,
    text: Option<&str>,
) -> Result<(), ErrorResponse> {
    let from_addr = "No reply <no-reply@domain.tld>"
        .parse()
        .map_err(|_| generic_internal_error())?;
    let to_addr = to_email.parse().map_err(|_| generic_internal_error())?;

    let body_html = html.map(|s| s.to_string());
    let body_text = text.map(|s| s.to_string()).or_else(|| body_html.clone());

    let (content_type, body) = if let Some(h) = body_html {
        (ContentType::TEXT_HTML, h)
    } else if let Some(t) = body_text {
        (ContentType::TEXT_PLAIN, t)
    } else {
        (ContentType::TEXT_PLAIN, "".to_string())
    };

    let email = Message::builder()
        .from(from_addr)
        .to(to_addr)
        .subject(subject)
        .header(content_type)
        .body(body)
        .map_err(|_| generic_internal_error())?;

    mailer
        .send(email)
        .await
        .map_err(|_| generic_internal_error())?;
    Ok(())
}

#[debug_handler]
pub async fn subscribe(
    State(state): State<AppState>,
    payload: ValidatedJson<V1SubscribePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let email = payload.email.trim().to_lowercase();
    let token = Uuid::new_v4().to_string();

    // Abuse limiter: per-email subscription attempts
    let key = format!("newsletter:subscribe:{}", email);
    let config = AbuseLimiterConfig {
        temp_block_attempts: 5,
        temp_block_range: 60,
        temp_block_duration: 60 * 60,
        block_retry_limit: 20,
        block_range: 24 * 60 * 60,
        block_duration: 24 * 60 * 60,
    };
    limiter(&state.redis_pool, &key, config).await?;

    let new_sub = NewSubscriber {
        email: email.clone(),
        status: SubscriberStatus::Pending,
        token: token.clone(),
    };

    match SubscriberEntity::create(&state.sea_db, new_sub).await {
        Ok(_model) => {
            let site_url =
                std::env::var("SITE_URL").unwrap_or_else(|_| "http://localhost:8888".to_string());
            let confirm_url = format!(
                "{}/newsletter/confirm?email={}&token={}",
                site_url.trim_end_matches('/'),
                urlencoding::encode(&email),
                urlencoding::encode(&token)
            );

            let subject = "Confirm your subscription";
            let html = format!(
                "<p>Thanks for subscribing!</p><p>Please confirm your subscription by clicking the link below:</p><p><a href=\"{0}\">{0}</a></p>",
                confirm_url
            );
            // Best-effort email; do not fail subscription on send error
            let _ = send_mail(&state.mailer, &email, subject, Some(&html), None).await;

            #[allow(unused_mut)]
            let mut body =
                json!({ "message": "Please check your email to confirm your subscription" });
            #[cfg(debug_assertions)]
            {
                if let Some(obj) = body.as_object_mut() {
                    obj.insert(
                        "debug".to_string(),
                        json!({ "token": token, "confirm_url": confirm_url }),
                    );
                }
            }
            Ok((StatusCode::CREATED, Json(body)))
        }
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn unsubscribe(
    State(state): State<AppState>,
    payload: ValidatedJson<V1UnsubscribePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let email = payload.email.trim().to_lowercase();
    let token = payload.token.trim().to_string();

    match SubscriberEntity::unsubscribe(&state.sea_db, &email, Some(&token)).await {
        Ok(Some(_)) => Ok(Json(json!({ "message": "Unsubscribed successfully" }))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
            .with_message("Invalid token or subscriber not found")),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn send(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1SendNewsletterPayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    // Load confirmed subscribers
    let subscribers = SubscriberEntity::find()
        .filter(SubscriberColumn::Status.eq(SubscriberStatus::Confirmed))
        .all(&state.sea_db)
        .await?;

    let mut sent_count: u64 = 0;
    for sub in subscribers {
        let to = sub.email.as_str();
        // Try best-effort; continue on failure
        if send_mail(
            &state.mailer,
            to,
            &payload.subject,
            payload.html.as_deref(),
            Some(&payload.text),
        )
        .await
        .is_ok()
        {
            sent_count += 1;
        }
    }

    Ok(Json(json!({
        "message": "Newsletter sent",
        "sent": sent_count
    })))
}

#[debug_handler]
pub async fn list_subscribers(
    State(state): State<AppState>,
    _auth: AuthSession,
    payload: ValidatedJson<V1ListSubscribersQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let query = SubscriberQuery {
        page_no: payload.page,
        search: payload.search.clone(),
        status: None,
        sort_by: None,
        sort_order: None,
    };

    match SubscriberEntity::find_with_query(&state.sea_db, query).await {
        Ok((items, total)) => Ok(Json(json!({
            "data": items,
            "total": total,
            "page": payload.page_or_default()
        }))),
        Err(err) => Err(err.into()),
    }
}

#[debug_handler]
pub async fn confirm(
    State(state): State<AppState>,
    payload: ValidatedJson<V1UnsubscribePayload>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let email = payload.email.trim().to_lowercase();
    let token = payload.token.trim().to_string();

    match SubscriberEntity::confirm(&state.sea_db, &email, &token).await {
        Ok(Some(_)) => Ok(Json(json!({ "message": "Subscription confirmed" }))),
        Ok(None) => Err(ErrorResponse::new(ErrorCode::OperationNotAllowed)
            .with_message("Invalid token or subscriber not found")),
        Err(err) => Err(err.into()),
    }
}
