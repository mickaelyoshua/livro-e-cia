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
