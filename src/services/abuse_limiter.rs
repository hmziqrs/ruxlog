use fred::{
    prelude::{KeysInterface, Pool as RedisPool, SortedSetsInterface},
    types::Expiration,
};

use crate::error::{ErrorCode, ErrorResponse};

pub struct AbuseLimiterConfig {
    pub temp_block_attempts: usize,
    pub temp_block_range: usize,
    pub temp_block_duration: usize,
    pub block_retry_limit: usize,
    pub block_range: usize,
    pub block_duration: usize,
}

/// Implements an abuse limiter to prevent excessive attempts at a specific action.
///
/// # Arguments
///
/// * `redis_pool` - A reference to the Redis connection pool.
/// * `key_prefix` - A string prefix used to create unique Redis keys for this limiter.
/// * `config` - An `AbuseLimiterConfig` struct containing the limiter's configuration parameters.
///
/// # Returns
///
/// * `Ok(())` if the action is allowed to proceed.
/// * `Err(ErrorResponse)` if the action is blocked, containing a standardized error response.
///
/// # Functionality
///
/// This function implements two levels of blocking:
/// 1. A temporary block for exceeding attempts within a short time frame.
/// 2. A longer block for exceeding attempts within a longer time frame.
///
/// It uses Redis to track attempts and manage blocking durations.
pub async fn limiter(
    redis_pool: &RedisPool,
    key_prefix: &str,
    config: AbuseLimiterConfig,
) -> Result<(), ErrorResponse> {
    let block_key = format!("abuse_limiter:block:{}", key_prefix);
    let is_blocked: Option<String> = redis_pool.get(&block_key).await.unwrap();
    if let Some(blocked_until) = is_blocked {
        let blocked_until: usize = blocked_until.parse().unwrap();
        let current_time = chrono::Utc::now().timestamp() as usize;
        if current_time < blocked_until {
            return Err(ErrorResponse::new(ErrorCode::TooManyAttempts)
                .with_message("You have been temporarily blocked due to too many verification attempts. Please try again later"));
        } else {
            let _: () = redis_pool.del(&block_key).await.unwrap();
        }
    }

    // Track the number of attempts
    let attempt_key = format!("abuse_limiter:attempts:{}", key_prefix);
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
        .expire(&attempt_key, config.block_duration as i64, None)
        .await
        .unwrap();

    let temp_block_attempts: usize = redis_pool
        .zcount(
            &attempt_key,
            (current_time - config.temp_block_range) as f64,
            current_time as f64,
        )
        .await
        .unwrap();
    let block_attempts: usize = redis_pool
        .zcount(
            &attempt_key,
            (current_time - config.block_range) as f64,
            current_time as f64,
        )
        .await
        .unwrap();

    if temp_block_attempts > config.temp_block_attempts {
        let block_until = current_time + config.temp_block_duration;
        let _: () = redis_pool
            .set(
                &block_key,
                block_until as i64,
                Some(Expiration::EX(config.temp_block_duration as i64)),
                None,
                false,
            )
            .await
            .unwrap();
        return Err(ErrorResponse::new(ErrorCode::TooManyAttempts)
            .with_message("You have been temporarily blocked due to too many verification attempts. Please try again in an hour."));
    }

    if block_attempts > config.block_retry_limit {
        let block_until = current_time + config.block_duration;
        let _: () = redis_pool
            .set(
                &block_key,
                block_until as i64,
                Some(Expiration::EX(config.block_duration as i64)),
                None,
                false,
            )
            .await
            .unwrap();
        return Err(ErrorResponse::new(ErrorCode::TooManyAttempts)
            .with_message("You have been temporarily blocked due to too many verification attempts. Please try again in 24 hours."));
    }

    Ok(())
}
