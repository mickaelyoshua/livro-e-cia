use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request, State,
};

use crate::{
    config::Environment,
    error::ApiError,
    rate_limit::{check_rate_limit, RateLimitConfig},
    redis::RedisPool,
};

// Extract client IP (handles reverse proxy headers)
fn get_client_ip(req: &Request<'_>) -> String {
    req.headers()
        .get_one("X-Forwarded-For")
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| req.headers().get_one("X-Real-IP").map(String::from))
        .or_else(|| req.client_ip().map(|ip| ip.to_string()))
        .unwrap_or_else(|| "unknown".to_string())
}

pub struct RateLimitedLogin;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimitedLogin {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let environment = match req.guard::<&State<Environment>>().await {
            Outcome::Success(e) => *e.inner(),
            _ => Environment::Development,
        };

        let redis = match req.guard::<&State<RedisPool>>().await {
            Outcome::Success(r) => r,
            _ => {
                log::warn!("Redis unavailable, allowing request (fail-open)");
                return Outcome::Success(RateLimitedLogin);
            }
        };

        let ip = get_client_ip(req);
        let mut redis_conn = redis.inner().clone();
        let config = RateLimitConfig::login(environment);

        match check_rate_limit(&mut redis_conn, &config, &ip).await {
            Ok(result) if result.allowed => Outcome::Success(RateLimitedLogin),
            Ok(result) => {
                log::warn!("Rate limit exceeded for login from IP: {}", ip);
                Outcome::Error((
                    Status::TooManyRequests,
                    ApiError::RateLimitExceeded {
                        retry_after_seconds: result
                            .retry_after_seconds
                            .unwrap_or(config.window_seconds),
                    },
                ))
            }
            Err(e) => {
                log::error!("Rate limit check failed (fail-open): {}", e);
                Outcome::Success(RateLimitedLogin)
            }
        }
    }
}

pub struct RateLimitedForgotPassword;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimitedForgotPassword {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let environment = match req.guard::<&State<Environment>>().await {
            Outcome::Success(e) => *e.inner(),
            _ => Environment::Development,
        };

        let redis = match req.guard::<&State<RedisPool>>().await {
            Outcome::Success(r) => r,
            _ => {
                log::warn!("Redis unavailable, allowing request (fail-open)");
                return Outcome::Success(RateLimitedForgotPassword);
            }
        };

        let ip = get_client_ip(req);
        let mut redis_conn = redis.inner().clone();
        let config = RateLimitConfig::login(environment);

        match check_rate_limit(&mut redis_conn, &config, &ip).await {
            Ok(result) if result.allowed => Outcome::Success(RateLimitedForgotPassword),
            Ok(result) => {
                log::warn!("Rate limit exceeded for forgot password from IP: {}", ip);
                Outcome::Error((
                    Status::TooManyRequests,
                    ApiError::RateLimitExceeded {
                        retry_after_seconds: result
                            .retry_after_seconds
                            .unwrap_or(config.window_seconds),
                    },
                ))
            }
            Err(e) => {
                log::error!("Rate limit check failed (fail-open): {}", e);
                Outcome::Success(RateLimitedForgotPassword)
            }
        }
    }
}

pub struct RateLimitedResetPassword;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimitedResetPassword {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let environment = match req.guard::<&State<Environment>>().await {
            Outcome::Success(e) => *e.inner(),
            _ => Environment::Development,
        };

        let redis = match req.guard::<&State<RedisPool>>().await {
            Outcome::Success(r) => r,
            _ => {
                log::warn!("Redis unavailable, allowing request (fail-open)");
                return Outcome::Success(RateLimitedResetPassword);
            }
        };

        let ip = get_client_ip(req);
        let mut redis_conn = redis.inner().clone();
        let config = RateLimitConfig::login(environment);

        match check_rate_limit(&mut redis_conn, &config, &ip).await {
            Ok(result) if result.allowed => Outcome::Success(RateLimitedResetPassword),
            Ok(result) => {
                log::warn!("Rate limit exceeded for reset password from IP: {}", ip);
                Outcome::Error((
                    Status::TooManyRequests,
                    ApiError::RateLimitExceeded {
                        retry_after_seconds: result
                            .retry_after_seconds
                            .unwrap_or(config.window_seconds),
                    },
                ))
            }
            Err(e) => {
                log::error!("Rate limit check failed (fail-open): {}", e);
                Outcome::Success(RateLimitedResetPassword)
            }
        }
    }
}

pub struct RateLimitedVerifyEmail;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimitedVerifyEmail {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let environment = match req.guard::<&State<Environment>>().await {
            Outcome::Success(e) => *e.inner(),
            _ => Environment::Development,
        };

        let redis = match req.guard::<&State<RedisPool>>().await {
            Outcome::Success(r) => r,
            _ => {
                log::warn!("Redis unavailable, allowing request (fail-open)");
                return Outcome::Success(RateLimitedVerifyEmail);
            }
        };

        let ip = get_client_ip(req);
        let mut redis_conn = redis.inner().clone();
        let config = RateLimitConfig::login(environment);

        match check_rate_limit(&mut redis_conn, &config, &ip).await {
            Ok(result) if result.allowed => Outcome::Success(RateLimitedVerifyEmail),
            Ok(result) => {
                log::warn!("Rate limit exceeded for verify email from IP: {}", ip);
                Outcome::Error((
                    Status::TooManyRequests,
                    ApiError::RateLimitExceeded {
                        retry_after_seconds: result
                            .retry_after_seconds
                            .unwrap_or(config.window_seconds),
                    },
                ))
            }
            Err(e) => {
                log::error!("Rate limit check failed (fail-open): {}", e);
                Outcome::Success(RateLimitedVerifyEmail)
            }
        }
    }
}
