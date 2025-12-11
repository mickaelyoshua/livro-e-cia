use crate::config::Environment;

pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub key_prefix: &'static str,
}

impl RateLimitConfig {
    pub fn login(env: Environment) -> Self {
        match env {
            Environment::Production => Self {
                max_requests: 5,
                window_seconds: 60,
                key_prefix: "rl:login",
            },
            Environment::Staging => Self {
                max_requests: 10,
                window_seconds: 60,
                key_prefix: "rl:login",
            },
            Environment::Development => Self {
                max_requests: 100,
                window_seconds: 60,
                key_prefix: "rl:login",
            },
        }
    }

    pub fn forgot_password(env: Environment) -> Self {
        match env {
            Environment::Production | Environment::Staging => Self {
                max_requests: 3,
                window_seconds: 3600,
                key_prefix: "rl:forgot_pwd",
            },
            Environment::Development => Self {
                max_requests: 50,
                window_seconds: 60,
                key_prefix: "rl:forgot_pwd",
            },
        }
    }

    pub fn reset_password(env: Environment) -> Self {
        match env {
            Environment::Production | Environment::Staging => Self {
                max_requests: 5,
                window_seconds: 60,
                key_prefix: "rl:reset_pwd",
            },
            Environment::Development => Self {
                max_requests: 50,
                window_seconds: 60,
                key_prefix: "rl:reset_pwd",
            },
        }
    }

    pub fn verify_email(env: Environment) -> Self {
        match env {
            Environment::Production | Environment::Staging => Self {
                max_requests: 10,
                window_seconds: 60,
                key_prefix: "rl:verify",
            },
            Environment::Development => Self {
                max_requests: 100,
                window_seconds: 60,
                key_prefix: "rl:verify",
            },
        }
    }
}
