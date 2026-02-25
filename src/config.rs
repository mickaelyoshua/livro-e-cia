use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

impl AppEnv {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            _ => Self::Development,
        }
    }
}

pub struct AppConfig {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub cookie_secret_key: String,
    pub app_env: AppEnv,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        assert!(
            jwt_secret.len() >= 32,
            "JWT_SECRET must be at least 32 bytes"
        );

        let cookie_secret_key =
            env::var("COOKIE_SECRET_KEY").expect("COOKIE_SECRET_KEY must be set");
        assert!(
            cookie_secret_key.len() >= 64,
            "COOKIE_SECRET_KEY must be at least 64 bytes"
        );

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("PORT must be a valid u16"),
            jwt_secret,
            cookie_secret_key,
            app_env: AppEnv::from_str(
                &env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            ),
        }
    }

    pub fn is_production(&self) -> bool {
        self.app_env == AppEnv::Production
    }
}
