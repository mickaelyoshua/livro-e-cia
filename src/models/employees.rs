use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::schema::employees)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::employees)]
pub struct NewEmployee {
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::employees)]
pub struct UpdateEmployee {
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub name: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}
