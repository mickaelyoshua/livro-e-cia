use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))] // Compile time safety check from Diesel
                                             // To match at compile time the model with the PostgreSQL table
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::roles)]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
}

impl Role {
    pub fn into_dto(self) -> shared::RoleDto {
        shared::RoleDto {
            id: self.id,
            name: self.name,
            description: self.description,
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
            name: "admin".to_string(),
            description: Some("Administrator".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn into_dto_maps_id_and_name() {
        let role = sample_role();
        let id = role.id;
        let dto = role.into_dto();
        assert_eq!(dto.id, id);
        assert_eq!(dto.name, "admin");
    }

    #[test]
    fn into_dto_maps_description() {
        let role = sample_role();
        let dto = role.into_dto();
        assert_eq!(dto.description, Some("Administrator".to_string()));
    }

    #[test]
    fn into_dto_handles_none_description() {
        let mut role = sample_role();
        role.description = None;
        let dto = role.into_dto();
        assert!(dto.description.is_none());
    }

    #[test]
    fn into_dto_omits_timestamps() {
        // Compile-time verification: RoleDto has no created_at/updated_at
        let role = sample_role();
        let _dto: shared::RoleDto = role.into_dto();
    }
}
