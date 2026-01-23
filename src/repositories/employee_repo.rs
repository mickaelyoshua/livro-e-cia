use uuid::Uuid;

use crate::db::DbConn;
use crate::error::AppError;
use crate::models::{Employee, NewEmployee, UpdateEmployee};

pub fn find_by_email(conn: &mut DbConn, email: &str) -> Result<Option<Employee>, AppError> {
    unimplemented!()
}

pub fn find_by_id(conn: &mut DbConn, id: Uuid) -> Result<Option<Employee>, AppError> {
    unimplemented!()
}

pub fn list_all(conn: &mut DbConn) -> Result<Vec<Employee>, AppError> {
    unimplemented!()
}

pub fn create(conn: &mut DbConn, new_employee: NewEmployee) -> Result<Employee, AppError> {
    unimplemented!()
}

pub fn update(conn: &mut DbConn, id: Uuid, update: UpdateEmployee) -> Result<Employee, AppError> {
    unimplemented!()
}

pub fn soft_delete(conn: &mut DbConn, id: Uuid) -> Result<(), AppError> {
    unimplemented!()
}
