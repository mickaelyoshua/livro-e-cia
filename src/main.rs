mod config;
mod error;
mod models;

use std::net::SocketAddr;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("Connected to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations applied");

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    let addr = SocketAddr::new(config.host.parse().expect("Invalid HOST address"), config.port);
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("Listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    tracing::info!("Shutdown signal received");
}
