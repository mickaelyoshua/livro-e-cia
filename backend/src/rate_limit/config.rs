pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub key_prefix: &'static str,
}

impl RateLimitConfig {
    pub const LOGIN: Self = Self {
        max_requests: 5,
        window_seconds: 60,
        key_prefix: "login",
    };

    pub const FORGOT_PASSWORD: Self = Self {
        max_requests: 3,
        window_seconds: 3600, // 1 hour
        key_prefix: "forgot_password",
    };

    pub const RESET_PASSWORD: Self = Self {
        max_requests: 5,
        window_seconds: 60,
        key_prefix: "reset_password",
    };

    pub const VERIFY_EMAIL: Self = Self {
        max_requests: 10,
        window_seconds: 60,
        key_prefix: "verify_email",
    };
}
