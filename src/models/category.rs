use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
}

pub struct UpdateCategory {
    pub name: Option<String>,
    pub description: Option<String>,
}
