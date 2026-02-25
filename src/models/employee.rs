use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Employee {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewEmployee {
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
}

pub struct UpdateEmployee {
    pub email: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}
