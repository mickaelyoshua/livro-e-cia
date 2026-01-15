use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::models::PaymentMethod;

#[derive(Serialize)]
pub struct EmployeeView {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct SaleView {
    pub id: Uuid,
    pub seller_name: String,
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    pub payment_method: PaymentMethod,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub item_count: usize,
}
