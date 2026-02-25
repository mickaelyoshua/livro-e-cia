use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Product {
    pub id: Uuid,
    pub title: String,
    pub author: Option<String>,
    pub price: Decimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewProduct {
    pub title: String,
    pub author: Option<String>,
    pub price: Decimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
}

pub struct UpdateProduct {
    pub title: Option<String>,
    pub author: Option<String>,
    pub price: Option<Decimal>,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Option<Uuid>,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub is_active: Option<bool>,
}
