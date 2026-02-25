use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct SaleItem {
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
}

pub struct NewSaleItem {
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
}
