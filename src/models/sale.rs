use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use super::payment_method::PaymentMethod;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Sale {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    pub payment_method: PaymentMethod,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewSale {
    pub seller_id: Uuid,
    pub subtotal: Decimal,
    pub discount: Decimal,
    pub total: Decimal,
    pub payment_method: PaymentMethod,
    pub notes: Option<String>,
}
