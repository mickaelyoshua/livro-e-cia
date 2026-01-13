use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable, Associations, Identifiable)]
#[diesel(belongs_to(crate::models::Sale, foreign_key = sale_id))]
#[diesel(belongs_to(crate::models::Product, foreign_key = product_id))]
#[diesel(table_name = crate::schema::sale_items)]
#[diesel(primary_key(sale_id, product_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SaleItem {
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: rust_decimal::Decimal,
    pub subtotal: rust_decimal::Decimal,
}
