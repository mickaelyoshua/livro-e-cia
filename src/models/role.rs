use serde::Serialize;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Role {
    pub name: String,
    pub description: Option<String>,
}
