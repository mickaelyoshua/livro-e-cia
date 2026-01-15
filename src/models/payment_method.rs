use std::io::Write;

use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    serialize::{self, IsNull, ToSql},
};
use rocket::FromFormField;
use serde::Serialize;

#[derive(Debug, PartialEq, AsExpression, FromSqlRow, FromFormField, Serialize)]
#[diesel(sql_type = crate::schema::sql_types::PaymentMethod)]
pub enum PaymentMethod {
    #[field(value = "cash")] // From "FromFormField" to be used on the CreateSaleForm struct
    Cash,
    #[field(value = "credit_card")]
    CreditCard,
    #[field(value = "debit_card")]
    DebitCard,
    #[field(value = "pix")]
    Pix,
}

impl ToSql<crate::schema::sql_types::PaymentMethod, Pg> for PaymentMethod {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            PaymentMethod::Cash => out.write_all(b"cash")?,
            PaymentMethod::CreditCard => out.write_all(b"credit_card")?,
            PaymentMethod::DebitCard => out.write_all(b"debit_card")?,
            PaymentMethod::Pix => out.write_all(b"pix")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::PaymentMethod, Pg> for PaymentMethod {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"cash" => Ok(PaymentMethod::Cash),
            b"credit_card" => Ok(PaymentMethod::CreditCard),
            b"debit_card" => Ok(PaymentMethod::DebitCard),
            b"pix" => Ok(PaymentMethod::Pix),
            _ => Err("Unrecognized payment_method variant".into()),
        }
    }
}
