use std::env;

use dotenv::dotenv;
use rocket::{get, launch};

use crate::db::pool::DbConnection;

mod db;
mod models;
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

    let pool =
        db::pool::init_pool(&database_url).expect("Failed to create database connection pool");

    // Enables logging, so we can use log::tace!, log::debug!, log::error! ...
    env_logger::init();

    // stores pool in rocket state
    rocket::build().manage(pool).mount("/", rocket::routes![])
}
