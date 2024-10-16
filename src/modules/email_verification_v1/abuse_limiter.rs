use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use fred::{
    prelude::{KeysInterface, RedisPool, SortedSetsInterface},
    types::Expiration,
};
use serde_json::json;

const ATTEMPT_LIMIT_6_MINUTES: usize = 3;
const ATTEMPT_LIMIT_15_MINUTES: usize = 5;
const BLOCK_DURATION_1_HOUR: usize = 3600; // in seconds
const BLOCK_DURATION_24_HOURS: usize = 86400; // in seconds

pub async fn email_abuse_limiter(redis_pool: &RedisPool, user_id: &i32) -> Result<(), Response> {
    let block_key = format!("user:{}:email_abuse", user_id);
    let is_blocked: Option<String> = redis_pool.get(&block_key).await.unwrap();
    if let Some(blocked_until) = is_blocked {
        let blocked_until: usize = blocked_until.parse().unwrap();
        let current_time = chrono::Utc::now().timestamp() as usize;
        if current_time < blocked_until {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": "Too many attempts",
                    "message": "You have been temporarily blocked due to too many verification attempts. Please try again later",
                })),
            ).into_response());
        } else {
            // Unblock the user if the block duration has passed
            let _: () = redis_pool.del(&block_key).await.unwrap();
        }
    }

    // Track the number of attempts
    let attempt_key = format!("user:{}:attempts", user_id);
    let current_time = chrono::Utc::now().timestamp() as usize;
    let _: () = redis_pool
        .zadd(
            &attempt_key,
            None,
            None,
            false,
            false,
            (current_time as f64, current_time as f64),
        )
        .await
        .unwrap();
    let _: () = redis_pool
        .expire(&attempt_key, BLOCK_DURATION_24_HOURS as i64)
        .await
        .unwrap();

    // Get the number of attempts in the last 6 minutes and 15 minutes
    let attempts_6_minutes: usize = redis_pool
        .zcount(
            &attempt_key,
            (current_time - 360) as f64,
            current_time as f64,
        )
        .await
        .unwrap();
    let attempts_15_minutes: usize = redis_pool
        .zcount(
            &attempt_key,
            (current_time - 900) as f64,
            current_time as f64,
        )
        .await
        .unwrap();

    // Block the user if the limits are exceeded
    if attempts_6_minutes > ATTEMPT_LIMIT_6_MINUTES {
        let block_until = current_time + BLOCK_DURATION_1_HOUR;
        let _: () = redis_pool
            .set(
                &block_key,
                block_until as i64,
                Some(Expiration::EX(BLOCK_DURATION_1_HOUR as i64)),
                None,
                false,
            )
            .await
            .unwrap();
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "Too many attempts",
                "message": "You have been temporarily blocked due to too many verification attempts. Please try again in an hour.",
            })),
        ).into_response());
    }

    if attempts_15_minutes > ATTEMPT_LIMIT_15_MINUTES {
        let block_until = current_time + BLOCK_DURATION_24_HOURS;
        let _: () = redis_pool
            .set(
                &block_key,
                block_until as i64,
                Some(Expiration::EX(BLOCK_DURATION_24_HOURS as i64)),
                None,
                false,
            )
            .await
            .unwrap();
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "Too many attempts",
                "message": "You have been temporarily blocked due to too many verification attempts. Please try again in 24 hours.",
            })),
        ).into_response());
    }

    Ok(())
}
