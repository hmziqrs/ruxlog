use tower_sessions_redis_store::fred::interfaces::LuaInterface;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;
use tower_sessions_redis_store::fred::types::{FromValue, Value};
use tracing::{debug, error, info, instrument, warn};

use crate::error::{ErrorCode, ErrorResponse};

#[derive(Clone, Copy, Debug)]
pub struct AbuseLimiterConfig {
    pub temp_block_attempts: usize,
    pub temp_block_range: usize,    // seconds
    pub temp_block_duration: usize, // seconds
    pub block_retry_limit: usize,   // long threshold
    pub block_range: usize,         // seconds
    pub block_duration: usize,      // seconds
}

#[derive(Debug, Clone, Copy)]
pub enum BlockScope {
    Temp,
    Long,
}

#[derive(Debug, Clone)]
pub enum LimiterDecision {
    Allowed {
        short_count: u64,
        long_count: u64,
    },
    Blocked {
        scope: BlockScope,
        retry_after_secs: u64,
        short_count: u64,
        long_count: u64,
    },
}

const LUA_SCRIPT: &str = r#"
-- KEYS: attempts_key, block_key, seq_key
-- ARGV: temp_window, temp_threshold, temp_block_duration, long_window, long_threshold, long_block_duration, attempts_ttl

local attempts_key = KEYS[1]
local block_key = KEYS[2]
local seq_key = KEYS[3]

local temp_window = tonumber(ARGV[1])
local temp_threshold = tonumber(ARGV[2])
local temp_block_duration = tonumber(ARGV[3])
local long_window = tonumber(ARGV[4])
local long_threshold = tonumber(ARGV[5])
local long_block_duration = tonumber(ARGV[6])
local attempts_ttl = tonumber(ARGV[7])

local now = redis.call('TIME')
local now_sec = tonumber(now[1])

-- If already blocked, return the remaining TTL immediately
local existing_ttl = redis.call('TTL', block_key)
if existing_ttl and existing_ttl > 0 then
  -- Maintain attempts bookkeeping (optional): push the attempt but do not affect block state
  local seq = redis.call('INCR', seq_key)
  redis.call('EXPIRE', seq_key, attempts_ttl)
  local member = string.format('%d:%d', now_sec, seq)
  redis.call('ZADD', attempts_key, now_sec, member)
  redis.call('EXPIRE', attempts_key, attempts_ttl)
  local short_count = redis.call('ZCOUNT', attempts_key, now_sec - temp_window, now_sec)
  local long_count  = redis.call('ZCOUNT', attempts_key, now_sec - long_window, now_sec)
  return {0, existing_ttl, short_count, long_count, 'existing'}
end

local max_window = math.max(temp_window, long_window)
redis.call('ZREMRANGEBYSCORE', attempts_key, '-inf', now_sec - max_window)

local seq = redis.call('INCR', seq_key)
redis.call('EXPIRE', seq_key, attempts_ttl)
local member = string.format('%d:%d', now_sec, seq)
redis.call('ZADD', attempts_key, now_sec, member)
redis.call('EXPIRE', attempts_key, attempts_ttl)

local short_count = redis.call('ZCOUNT', attempts_key, now_sec - temp_window, now_sec)
local long_count  = redis.call('ZCOUNT', attempts_key, now_sec - long_window, now_sec)

if short_count >= temp_threshold then
  redis.call('SET', block_key, '1', 'EX', temp_block_duration, 'NX')
  local ttl = redis.call('TTL', block_key)
  if ttl < 0 then ttl = 0 end
  return {0, ttl, short_count, long_count, 'temp'}
elseif long_count >= long_threshold then
  redis.call('SET', block_key, '1', 'EX', long_block_duration, 'NX')
  local ttl = redis.call('TTL', block_key)
  if ttl < 0 then ttl = 0 end
  return {0, ttl, short_count, long_count, 'long'}
else
  return {1, 0, short_count, long_count, 'none'}
end
"#;

// Helpers: convert fred Value into primitives using FromValue.
#[inline]
fn to_u64(v: &Value) -> Option<u64> {
    u64::from_value(v.clone()).ok()
}
#[inline]
fn to_string(v: &Value) -> Option<String> {
    String::from_value(v.clone()).ok()
}

/// Execute the limiter in Redis via a single atomic Lua script.
#[instrument(skip(redis_pool), fields(
    scope = %key_prefix,
    decision,
    short_count,
    long_count,
    retry_after
))]
pub async fn check(
    redis_pool: &RedisPool,
    key_prefix: &str,
    config: AbuseLimiterConfig,
) -> Result<LimiterDecision, ErrorResponse> {
    debug!(
        temp_threshold = config.temp_block_attempts,
        temp_window = config.temp_block_range,
        long_threshold = config.block_retry_limit,
        long_window = config.block_range,
        "Checking abuse limiter"
    );
    let attempts_key = format!("abuse_limiter:attempts:{}", key_prefix);
    let block_key = format!("abuse_limiter:block:{}", key_prefix);
    let seq_key = format!("abuse_limiter:seq:{}", key_prefix);

    let attempts_ttl = std::cmp::max(config.temp_block_range, config.block_range) + 60; // slack 60s

    let keys = vec![attempts_key, block_key, seq_key];
    // fred 10 expects args TryInto<MultipleValues>. Vec<Value> is supported.
    let args: Vec<Value> = vec![
        Value::from(config.temp_block_range as i64),
        Value::from(config.temp_block_attempts as i64),
        Value::from(config.temp_block_duration as i64),
        Value::from(config.block_range as i64),
        Value::from(config.block_retry_limit as i64),
        Value::from(config.block_duration as i64),
        Value::from(attempts_ttl as i64),
    ];

    // Evaluate the script directly via the Pool. This avoids explicit SCRIPT LOAD.
    let res: Result<Vec<Value>, _> = redis_pool.eval(LUA_SCRIPT, keys, args).await;
    let values = match res {
        Ok(v) => v,
        Err(err) => {
            error!(
                error = %err,
                key_prefix = %key_prefix,
                "Redis error during limiter check"
            );
            return Err(ErrorResponse::new(ErrorCode::ServiceUnavailable)
                .with_message("Limiter unavailable (Redis error)")
                .with_details(err.to_string()));
        }
    };

    if values.len() != 5 {
        error!(
            value_count = values.len(),
            key_prefix = %key_prefix,
            "Unexpected Lua script result length"
        );
        return Err(ErrorResponse::new(ErrorCode::InternalServerError)
            .with_message("Limiter returned unexpected result"));
    }

    let allowed = to_u64(&values[0]).unwrap_or(0);
    let retry_after = to_u64(&values[1]).unwrap_or(0);
    let short_count = to_u64(&values[2]).unwrap_or(0);
    let long_count = to_u64(&values[3]).unwrap_or(0);
    let reason = to_string(&values[4]).unwrap_or_else(|| "none".to_string());

    if allowed == 1 {
        debug!(short_count, long_count, "Request allowed");
        tracing::Span::current().record("decision", "allowed");
        tracing::Span::current().record("short_count", short_count);
        tracing::Span::current().record("long_count", long_count);

        return Ok(LimiterDecision::Allowed {
            short_count,
            long_count,
        });
    }

    let scope = match reason.as_str() {
        "temp" => BlockScope::Temp,
        "long" => BlockScope::Long,
        _ => BlockScope::Temp,
    };

    warn!(
        scope = ?scope,
        retry_after = retry_after,
        short_count,
        long_count,
        reason = %reason,
        "Request blocked by abuse limiter"
    );

    tracing::Span::current().record("decision", "blocked");
    tracing::Span::current().record("short_count", short_count);
    tracing::Span::current().record("long_count", long_count);
    tracing::Span::current().record("retry_after", retry_after);

    Ok(LimiterDecision::Blocked {
        scope,
        retry_after_secs: retry_after,
        short_count,
        long_count,
    })
}

/// Backward-compatible wrapper preserving the original signature.
#[instrument(skip(redis_pool), fields(scope = %key_prefix))]
pub async fn limiter(
    redis_pool: &RedisPool,
    key_prefix: &str,
    config: AbuseLimiterConfig,
) -> Result<(), ErrorResponse> {
    use serde_json::json;

    match check(redis_pool, key_prefix, config).await? {
        LimiterDecision::Allowed { .. } => {
            info!("Access allowed");
            Ok(())
        }
        LimiterDecision::Blocked {
            retry_after_secs, ..
        } => {
            info!(
                retry_after = retry_after_secs,
                "Access denied - rate limited"
            );
            Err(ErrorResponse::new(ErrorCode::TooManyAttempts)
                .with_message(format!(
                    "Too many attempts. Try again in {} seconds.",
                    retry_after_secs
                ))
                .with_retry_after(retry_after_secs)
                .with_context(json!({ "retryAfter": retry_after_secs })))
        }
    }
}
