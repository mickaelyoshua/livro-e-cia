use diesel::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(primary_key(name))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub name: String,
    pub description: Option<String>,
}
