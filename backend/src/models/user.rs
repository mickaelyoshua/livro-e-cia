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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_role() -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "employee".to_string(),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "$argon2id$secret_hash".to_string(),
            name: "Test User".to_string(),
            role_id: Uuid::new_v4(),
            is_active: true,
            email_verified: true,
            email_verified_at: Some(Utc::now()),
            password_reset_token: Some("reset_token".to_string()),
            password_reset_expires_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: Some(Utc::now()),
        }
    }

    #[test]
    fn into_dto_maps_basic_fields() {
        let user = sample_user();
        let id = user.id;
        let email = user.email.clone();
        let name = user.name.clone();
        let role = sample_role();
        let dto = user.into_dto(role);

        assert_eq!(dto.id, id);
        assert_eq!(dto.email, email);
        assert_eq!(dto.name, name);
        assert!(dto.is_active);
        assert!(dto.email_verified);
    }

    #[test]
    fn into_dto_embeds_role() {
        let user = sample_user();
        let role = sample_role();
        let role_name = role.name.clone();
        let dto = user.into_dto(role);

        assert_eq!(dto.role.name, role_name);
    }

    #[test]
    fn into_dto_excludes_password_hash() {
        // SECURITY: UserDto must NOT have password_hash field
        // This is a compile-time check - struct doesn't have the field
        let user = sample_user();
        let role = sample_role();
        let _dto: shared::UserDto = user.into_dto(role);
        // If this compiles, password_hash is not in UserDto
    }

    #[test]
    fn into_dto_excludes_sensitive_fields() {
        // These fields should NOT be in UserDto:
        // password_hash, password_reset_token, password_reset_expires_at,
        // email_verified_at, updated_at, role_id
        let user = sample_user();
        let role = sample_role();
        let _dto: shared::UserDto = user.into_dto(role);
    }

    #[test]
    fn into_dto_handles_none_last_login() {
        let mut user = sample_user();
        user.last_login_at = None;
        let role = sample_role();
        let dto = user.into_dto(role);

        assert!(dto.last_login_at.is_none());
    }

    #[test]
    fn into_dto_preserves_timestamps() {
        let user = sample_user();
        let created = user.created_at;
        let last_login = user.last_login_at;
        let role = sample_role();
        let dto = user.into_dto(role);

        assert_eq!(dto.created_at, created);
        assert_eq!(dto.last_login_at, last_login);
    }
}
