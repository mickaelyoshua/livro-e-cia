use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::categories)]
pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::categories)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
}

impl Category {
    pub fn into_dto(self) -> shared::CategoryDto {
        shared::CategoryDto {
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

    fn sample_category() -> Category {
        Category {
            id: Uuid::new_v4(),
            name: "Fiction".to_string(),
            description: Some("Fiction books".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn into_dto_maps_all_fields() {
        let category = sample_category();
        let id = category.id;
        let dto = category.into_dto();
        assert_eq!(dto.id, id);
        assert_eq!(dto.name, "Fiction");
        assert_eq!(dto.description, Some("Fiction books".to_string()));
    }

    #[test]
    fn into_dto_handles_none_description() {
        let mut category = sample_category();
        category.description = None;
        let dto = category.into_dto();
        assert!(dto.description.is_none());
    }

    #[test]
    fn into_dto_omits_timestamps() {
        // Compile-time verification: CategoryDto has no created_at/updated_at
        let category = sample_category();
        let _dto: shared::CategoryDto = category.into_dto();
    }
}
