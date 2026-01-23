use dotenvy::dotenv;
use rocket::launch;
use rocket_dyn_templates::Template;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{auth::JwtConfig, config::AppConfig, db::init_pool};

mod auth;
mod config;
mod db;
mod error;
mod models;
mod repositories;
mod routes;
mod schema;

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let conf = AppConfig::from_env();
    info!(is_production = %conf.is_production, "App configuration loaded");

    let pool = init_pool(&conf.database_url).expect("Failed to create database pool");

    let jwt_conf = JwtConfig::from_env().expect("Failed to load JWT configuration");

    info!("Start Livro e Cia server");

    rocket::build()
        .manage(pool)
        .manage(jwt_conf)
        .manage(conf)
        .attach(Template::fairing())
}
