use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::PaymentMethod;

#[derive(Debug, Queryable, Associations)]
#[diesel(belongs_to(crate::models::User, foreign_key = seller_id))]
#[diesel(table_name = crate::schema::sales)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Sale {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub subtotal: rust_decimal::Decimal,
    pub discount: rust_decimal::Decimal,
    pub total: rust_decimal::Decimal,
    pub payment_method: PaymentMethod,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
