use dotenv::dotenv;
use rocket::launch;

use crate::{
    config::AppConfig,
    email::{EmailConfig, EmailService, MockEmailService, SmtpConfig, SmtpEmailService},
    fairings::SecurityHeaders,
};

mod auth;
mod config;
mod db;
mod email;
mod error;
mod fairings;
mod models;
mod redis;
mod routes;
mod schema;
mod utils;

// Macro for Rocket application entry point
// Replace #[rocker::main]
// Set up async runtime
#[launch]
fn rocket() -> _ {
    // Load .env file
    dotenv().ok(); // the '.ok()' will turn the Result into a Option
                   // Production uses real env vars, so if there is an error loading the .env file, the code will
                   // continue since it will return None

    let config = AppConfig::from_env().unwrap_or_else(|e| {
        eprintln!("FATAL: configuration error: {}", e);
        std::process::exit(1);
    });

    let cors = config.cors().expect("Failed to create CORS configuration");

    let pool =
        db::init_pool(&config.database_url).expect("Failed to create database connection pool");

    let email_config = EmailConfig::from_env();
    let email_service: Box<dyn EmailService> = match std::env::var("EMAIL_PROVIDER")
        .unwrap_or_else(|_| "mock".to_string())
        .as_str()
    {
        "smtp" => {
            let smtp_config = SmtpConfig::from_env().unwrap_or_else(|e| {
                panic!(
                    "SMTP configuration required when EMAIL_PROVIDER=smtp: {}",
                    e
                );
            });
            Box::new(
                SmtpEmailService::new(email_config, smtp_config).unwrap_or_else(|e| {
                    panic!("Failed to initialize SMTP service: {}", e);
                }),
            )
        }
        _ => {
            log::info!("Using mock email service (emails logged to console)");
            Box::new(MockEmailService::new(email_config))
        }
    };

    // Enables logging, so we can use log::trace!, log::debug!, log::error! ...
    env_logger::init();

    // stores pool in rocket state
    rocket::build()
        .manage(config.jwt_secret)
        .manage(pool)
        .manage(email_service)
        .attach(cors)
        .attach(SecurityHeaders)
        .mount(
            "/",
            rocket::routes![
                routes::login,
                routes::get_current_user,
                routes::refresh_token,
                routes::logout,
                routes::list_products,
                routes::get_product,
                routes::create_product,
                routes::update_product,
                routes::delete_product,
                routes::list_employees,
                routes::get_employee,
                routes::create_employee,
                routes::update_employee,
                routes::delete_employee,
                routes::verify_email,
                routes::forgot_password,
                routes::reset_password,
            ],
        )
}
