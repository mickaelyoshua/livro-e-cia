use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    DebitCard,
    Pix,
}
