use chrono::NaiveDate;
use rocket::{
    FromForm,
    form::{self, FromFormField},
};
use uuid::Uuid;

use crate::models::PaymentMethod;

#[derive(Debug)]
pub struct FormDecimal(pub rust_decimal::Decimal);
// Necessary because rust_decimal::Decimal can not be directly used on struct form, trait limited
impl<'r> FromFormField<'r> for FormDecimal {
    fn from_value(field: form::ValueField<'r>) -> form::Result<'r, Self> {
        rust_decimal::Decimal::from_str_exact(field.value)
            .map(FormDecimal)
            .map_err(|_| form::Error::validation("invalid decimal").into())
    }
}

#[derive(Debug)]
pub struct FormNaiveDate(pub NaiveDate);
// Necessary because chrono::NaiveDate can not be directly used on struct form, trait limited
impl<'r> FromFormField<'r> for FormNaiveDate {
    fn from_value(field: form::ValueField<'r>) -> form::Result<'r, Self> {
        NaiveDate::parse_from_str(field.value, "%Y-%m-%d")
            .map(FormNaiveDate)
            .map_err(|_| form::Error::validation("invalid date, expected YYY-MM-DD").into())
    }
}

#[derive(Debug, FromForm)]
pub struct CreateUserForm {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role_id: Uuid,
}

#[derive(Debug, FromForm)]
pub struct UpdateUserForm {
    pub email: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
    pub role_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

#[derive(Debug, FromForm)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Debug, FromForm)]
pub struct CreateProductForm {
    pub title: String,
    pub author: String,
    pub price: FormDecimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<FormNaiveDate>,
    pub category_id: Uuid,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
}

#[derive(Debug, FromForm)]
pub struct UpdateProductForm {
    pub title: Option<String>,
    pub author: Option<String>,
    pub price: Option<FormDecimal>,
    pub stock_quantity: Option<i32>,
    pub publisher: Option<Option<String>>,
    pub publication_date: Option<Option<FormNaiveDate>>,
    pub category_id: Option<Uuid>,
    pub description: Option<Option<String>>,
    pub cover_image_url: Option<Option<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, FromForm)]
pub struct CreateSaleForm {
    pub seller_id: Uuid,
    pub subtotal: FormDecimal,
    pub discount: FormDecimal,
    pub total: FormDecimal,
    pub payment_method: PaymentMethod,
    pub notes: Option<String>,
}
