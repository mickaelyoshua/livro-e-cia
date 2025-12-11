use std::env;

use rocket_cors::{AllowedOrigins, CorsOptions};

const MIN_JWT_SECRET_LENGTH: usize = 32; // 256 bits

// ============================================
// Environment Detection
// ============================================

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match env::var("APP_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            _ => Environment::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }
}

// ============================================
// Configuration Errors
// ============================================

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(String),
    WeakJwtSecret { actual: usize, minimum: usize },
    ProductionSecurityViolation(String),
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
            ConfigError::ProductionSecurityViolation(msg) => {
                write!(f, "PRODUCTION SECURITY VIOLATION: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

// ============================================
// Application Configuration
// ============================================

pub struct AppConfig {
    pub environment: Environment,
    pub database_url: String,
    pub jwt_secret: String,
    pub cors_origins: Vec<String>,
    pub redis_url: String,
}

impl AppConfig {
    fn validate_production_config(database_url: &str) -> Result<(), ConfigError> {
        let has_ssl = database_url.contains("sslmode=require")
            || database_url.contains("sslmode=verify-full")
            || database_url.contains("sslmode=verify-ca");

        if !has_ssl {
            return Err(ConfigError::ProductionSecurityViolation(
                "DATABASE_URL must include sslmode=require in production".to_string(),
            ));
        }

        Ok(())
    }

    fn require_env(var: &str) -> Result<String, ConfigError> {
        env::var(var).map_err(|_| ConfigError::MissingEnvVar(var.to_string()))
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::from_env();

        let database_url = Self::require_env("DATABASE_URL")?;
        let jwt_secret = Self::require_env("JWT_SECRET")?;
        let redis_url = Self::require_env("REDIS_URL")?;

        if jwt_secret.len() < MIN_JWT_SECRET_LENGTH {
            return Err(ConfigError::WeakJwtSecret {
                actual: jwt_secret.len(),
                minimum: MIN_JWT_SECRET_LENGTH,
            });
        }

        if environment.is_production() {
            Self::validate_production_config(&database_url)?;
        }

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| {
                if environment.is_development() {
                    "http://localhost:8080".to_string()
                } else {
                    String::new()
                }
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        if environment.is_production() && cors_origins.is_empty() {
            return Err(ConfigError::ProductionSecurityViolation(
                "CORS_ORIGINS must be explicitly set in production".to_string(),
            ));
        }

        Ok(Self {
            environment,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn with_env_vars<F, R>(vars: &[(&str, &str)], f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let originals: Vec<_> = vars.iter().map(|(k, _)| (*k, env::var(k).ok())).collect();
        for (k, v) in vars {
            env::set_var(k, v);
        }
        let result = f();
        for (k, original) in originals {
            match original {
                Some(v) => env::set_var(k, v),
                None => env::remove_var(k),
            }
        }
        result
    }

    #[test]
    fn environment_from_env_defaults_to_development() {
        with_env_vars(&[], || {
            env::remove_var("APP_ENV");
            assert_eq!(Environment::from_env(), Environment::Development);
        });
    }

    #[test]
    fn environment_recognizes_production() {
        with_env_vars(&[("APP_ENV", "production")], || {
            assert_eq!(Environment::from_env(), Environment::Production);
        });
        with_env_vars(&[("APP_ENV", "prod")], || {
            assert_eq!(Environment::from_env(), Environment::Production);
        });
    }

    #[test]
    fn environment_recognizes_staging() {
        with_env_vars(&[("APP_ENV", "staging")], || {
            assert_eq!(Environment::from_env(), Environment::Staging);
        });
    }

    #[test]
    fn is_production_returns_correct_values() {
        assert!(Environment::Production.is_production());
        assert!(!Environment::Staging.is_production());
        assert!(!Environment::Development.is_production());
    }

    #[test]
    fn is_development_returns_correct_values() {
        assert!(Environment::Development.is_development());
        assert!(!Environment::Production.is_development());
    }

    #[test]
    fn config_rejects_short_jwt_secret() {
        with_env_vars(
            &[
                ("APP_ENV", "development"),
                ("DATABASE_URL", "postgres://test"),
                ("JWT_SECRET", "short"),
                ("REDIS_URL", "redis://localhost"),
            ],
            || {
                let result = AppConfig::from_env();
                assert!(matches!(result, Err(ConfigError::WeakJwtSecret { .. })));
            },
        );
    }

    #[test]
    fn config_accepts_32_byte_jwt_secret() {
        with_env_vars(
            &[
                ("APP_ENV", "development"),
                ("DATABASE_URL", "postgres://test"),
                ("JWT_SECRET", "12345678901234567890123456789012"),
                ("REDIS_URL", "redis://localhost"),
            ],
            || {
                assert!(AppConfig::from_env().is_ok());
            },
        );
    }

    #[test]
    fn production_requires_ssl_in_database_url() {
        with_env_vars(
            &[
                ("APP_ENV", "production"),
                ("DATABASE_URL", "postgres://user:pass@host/db"),
                ("JWT_SECRET", "12345678901234567890123456789012"),
                ("REDIS_URL", "redis://localhost"),
                ("CORS_ORIGINS", "https://example.com"),
            ],
            || {
                let result = AppConfig::from_env();
                assert!(matches!(result, Err(ConfigError::ProductionSecurityViolation(_))));
            },
        );
    }

    #[test]
    fn production_accepts_sslmode_require() {
        with_env_vars(
            &[
                ("APP_ENV", "production"),
                ("DATABASE_URL", "postgres://host/db?sslmode=require"),
                ("JWT_SECRET", "12345678901234567890123456789012"),
                ("REDIS_URL", "redis://localhost"),
                ("CORS_ORIGINS", "https://example.com"),
            ],
            || {
                assert!(AppConfig::from_env().is_ok());
            },
        );
    }

    #[test]
    fn production_requires_cors_origins() {
        with_env_vars(
            &[
                ("APP_ENV", "production"),
                ("DATABASE_URL", "postgres://host/db?sslmode=require"),
                ("JWT_SECRET", "12345678901234567890123456789012"),
                ("REDIS_URL", "redis://localhost"),
            ],
            || {
                env::remove_var("CORS_ORIGINS");
                let result = AppConfig::from_env();
                assert!(matches!(result, Err(ConfigError::ProductionSecurityViolation(_))));
            },
        );
    }
}
