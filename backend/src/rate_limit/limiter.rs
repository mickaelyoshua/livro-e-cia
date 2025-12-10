use chrono::Utc;
use thiserror::Error;

use crate::redis::RedisPool;

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
}

pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub retry_after_seconds: Option<u64>,
}

pub async fn check_rate_limit(
    redis: &mut RedisPool,
    config: &super::RateLimitConfig,
    identifier: &str,
) -> Result<RateLimitResult, RateLimitError> {
    let key = format!("rate_limit:{}:{}", config.key_prefix, identifier);

    let now_ms = Utc::now().timestamp_millis();
    let window_start_ms = now_ms - (config.window_seconds as i64 * 1000);

    // Atomic pipeline: cleanup old + add new + count + set TTL
    let (count,): (u32,) = redis::pipe()
        .atomic()
        .zrembyscore(&key, "-inf", window_start_ms)
        .ignore()
        .zadd(&key, now_ms, now_ms)
        .ignore()
        .zcard(&key)
        .expire(&key, config.window_seconds as i64 + 1)
        .ignore()
        .query_async(redis)
        .await?;

    if count > config.max_requests {
        Ok(RateLimitResult {
            allowed: false,
            remaining: 0,
            retry_after_seconds: Some(config.window_seconds),
        })
    } else {
        Ok(RateLimitResult {
            allowed: true,
            remaining: config.max_requests - count,
            retry_after_seconds: None,
        })
    }
}
