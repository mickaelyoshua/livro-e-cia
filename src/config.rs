use std::env;

pub struct AppConfig {
    pub is_production: bool,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let app_env = env::var("APP_ENV").unwrap_or_default();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

        Self {
            is_production: app_env.eq_ignore_ascii_case("production"),
            database_url,
        }
    }
}
