use crate::models::roles::Role;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(Role, foreign_key = role_id))]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role_id: Uuid,
    pub is_active: bool,
    pub email_verified: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role_id: Uuid,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::users)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub is_active: Option<bool>,
    pub email_verified: Option<bool>,
    // Nullable fields:
    // Outer Option will tell if the field should be skipped or not
    // Inner Option will tell to set the field to Null or to another value
    pub email_verified_at: Option<Option<DateTime<Utc>>>,
    pub password_reset_token: Option<Option<String>>,
    pub password_reset_expires_at: Option<Option<DateTime<Utc>>>,
    pub last_login_at: Option<Option<DateTime<Utc>>>,
}

impl User {
    pub fn into_dto(self, role: Role) -> shared::UserDto {
        shared::UserDto {
            id: self.id,
            email: self.email,
            name: self.name,
            role: role.into_dto(),
            is_active: self.is_active,
            email_verified: self.email_verified,
            created_at: self.created_at,
            last_login_at: self.last_login_at,
        }
    }
}
