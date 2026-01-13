use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable)]
#[diesel(table_name = crate::schema::categories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}
