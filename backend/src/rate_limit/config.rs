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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_production_allows_5_requests_per_minute() {
        let config = RateLimitConfig::login(Environment::Production);
        assert_eq!(config.max_requests, 5);
        assert_eq!(config.window_seconds, 60);
        assert_eq!(config.key_prefix, "rl:login");
    }

    #[test]
    fn login_staging_allows_10_requests_per_minute() {
        let config = RateLimitConfig::login(Environment::Staging);
        assert_eq!(config.max_requests, 10);
        assert_eq!(config.window_seconds, 60);
    }

    #[test]
    fn login_development_allows_100_requests_per_minute() {
        let config = RateLimitConfig::login(Environment::Development);
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window_seconds, 60);
    }

    #[test]
    fn forgot_password_production_allows_3_requests_per_hour() {
        let config = RateLimitConfig::forgot_password(Environment::Production);
        assert_eq!(config.max_requests, 3);
        assert_eq!(config.window_seconds, 3600);
        assert_eq!(config.key_prefix, "rl:forgot_pwd");
    }

    #[test]
    fn reset_password_production_allows_5_requests_per_minute() {
        let config = RateLimitConfig::reset_password(Environment::Production);
        assert_eq!(config.max_requests, 5);
        assert_eq!(config.window_seconds, 60);
        assert_eq!(config.key_prefix, "rl:reset_pwd");
    }

    #[test]
    fn verify_email_production_allows_10_requests_per_minute() {
        let config = RateLimitConfig::verify_email(Environment::Production);
        assert_eq!(config.max_requests, 10);
        assert_eq!(config.window_seconds, 60);
        assert_eq!(config.key_prefix, "rl:verify");
    }

    #[test]
    fn all_endpoints_have_unique_key_prefixes() {
        let login = RateLimitConfig::login(Environment::Production);
        let forgot = RateLimitConfig::forgot_password(Environment::Production);
        let reset = RateLimitConfig::reset_password(Environment::Production);
        let verify = RateLimitConfig::verify_email(Environment::Production);

        let prefixes = vec![login.key_prefix, forgot.key_prefix, reset.key_prefix, verify.key_prefix];
        let unique_count = prefixes.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, prefixes.len(), "All key prefixes must be unique");
    }

    #[test]
    fn production_is_more_restrictive_than_development() {
        let prod = RateLimitConfig::login(Environment::Production);
        let dev = RateLimitConfig::login(Environment::Development);
        assert!(prod.max_requests < dev.max_requests);
    }

    #[test]
    fn forgot_password_has_longer_window_than_login() {
        let login = RateLimitConfig::login(Environment::Production);
        let forgot = RateLimitConfig::forgot_password(Environment::Production);
        assert!(forgot.window_seconds > login.window_seconds);
    }
}
