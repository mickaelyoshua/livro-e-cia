use redis::aio::ConnectionManager;

// ConnectionManager auto-reconnect on failure, no manual pooling needed
pub type RedisPool = ConnectionManager;

pub async fn init_redis_pool(redis_url: &str) -> Result<RedisPool, redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    ConnectionManager::new(client).await
}
