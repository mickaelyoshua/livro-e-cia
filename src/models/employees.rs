use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable, Associations)]
#[diesel(belongs_to(crate::models::Role, foreign_key = role_id))]
#[diesel(table_name = crate::schema::employees)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Employee {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role_id: Uuid,
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
    pub role_id: Uuid,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::employees)]
pub struct UpdateEmployee {
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub name: Option<String>,
    pub role_id: Option<Uuid>,
    pub is_active: Option<bool>,
}
