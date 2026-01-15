use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable, Associations)]
#[diesel(belongs_to(crate::models::Category, foreign_key = category_id))]
#[diesel(table_name = crate::schema::products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Product {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub price: rust_decimal::Decimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Uuid,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::products)]
pub struct NewProduct {
    pub title: String,
    pub author: String,
    pub price: rust_decimal::Decimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Uuid,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::products)]
pub struct UpdateProduct {
    pub title: Option<String>,
    pub author: Option<String>,
    pub price: Option<rust_decimal::Decimal>,
    pub stock_quantity: Option<i32>,
    pub publisher: Option<Option<String>>,
    pub publication_date: Option<Option<NaiveDate>>,
    pub category_id: Option<Uuid>,
    pub description: Option<Option<String>>,
    pub cover_image_url: Option<Option<String>>,
    pub is_active: Option<bool>,
}
