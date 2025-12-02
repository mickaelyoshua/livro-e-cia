use std::env;

use dotenv::dotenv;
use rocket::launch;

mod auth;
mod db;
mod error;
mod models;
mod routes;
mod schema;

// Macro for Rocket application entry point
// Replace #[rocker::main]
// Set up async runtime
#[launch]
fn rocket() -> _ {
    // Load .env file
    dotenv().ok(); // the '.ok()' will turn the Result into a Option
                   // Production uses real env vars, so if there is an error loading the .env file, the code will
                   // continue since it will return None

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = db::init_pool(&database_url).expect("Failed to create database connection pool");

    // Enables logging, so we can use log::trace!, log::debug!, log::error! ...
    env_logger::init();

    // stores pool in rocket state
    rocket::build().manage(jwt_secret).manage(pool).mount(
        "/",
        rocket::routes![
            routes::login,
            routes::get_current_user,
            routes::refresh_token,
            routes::logout,
        ],
    )
}
