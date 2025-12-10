use std::env;

use rocket_cors::{AllowedOrigins, CorsOptions};

const MIN_JWT_SECRET_LENGTH: usize = 32; // 256 bits

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(String),
    WeakJwtSecret { actual: usize, minimum: usize },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => {
                write!(f, "Required environment variable '{}' is not set", var)
            }
            ConfigError::WeakJwtSecret { actual, minimum } => {
                write!(
                    f,
                    "JWT_SECRET is too weak: {} bytes (minimum: {} bytes). \n
                        Generate with: openssl rand -base64 32",
                    actual, minimum
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    pub redis_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?;

        if jwt_secret.len() < MIN_JWT_SECRET_LENGTH {
            return Err(ConfigError::WeakJwtSecret {
                actual: jwt_secret.len(),
                minimum: MIN_JWT_SECRET_LENGTH,
            });
        }

        let redis_url = env::var("REDIS_URL")
            .map_err(|_| ConfigError::MissingEnvVar("REDIS_URL".to_string()))?;

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:8080".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            database_url,
            jwt_secret,
            cors_origins,
            redis_url,
        })
    }

    pub fn cors(&self) -> Result<rocket_cors::Cors, rocket_cors::Error> {
        let allowed_origins = AllowedOrigins::some_exact(&self.cors_origins);

        CorsOptions {
            allowed_origins,
            allowed_methods: vec![
                rocket::http::Method::Get,
                rocket::http::Method::Post,
                rocket::http::Method::Put,
                rocket::http::Method::Delete,
                rocket::http::Method::Options,
            ]
            .into_iter()
            .map(From::from)
            .collect(),
            allowed_headers: rocket_cors::AllowedHeaders::some(&[
                "Authorization",
                "Content-Type",
                "Accept",
            ]),
            allow_credentials: true,
            max_age: Some(3600), // 1 hour. Alows to store in cache so the CORS response is not
            // always sent at every other request
            ..Default::default() // fill the rest with default value
        }
        .to_cors()
        // Converts to a Fairing: a middleware. In this case, the CORS fairing (rocket_cors::Cors)
    }
}
