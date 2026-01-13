use std::env;

use dotenvy::dotenv;

mod db;
mod models;

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    println!("{database_url}");
    println!("Hello, world!");
}
