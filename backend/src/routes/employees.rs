use diesel::prelude::*;
use rocket::{delete, get, http::Status, post, put, serde::json::Json, FromForm};
use serde_json::Value;
use shared::{CreateEmployeeRequest, PaginatedResponse, UpdateEmployeeRequest, UserDto};
use uuid::Uuid;

use crate::{
    auth::{hash_password, AdminGuard},
    db::DbConnection,
    error::ApiError,
    models::{NewUser, Role, UpdateUser, User},
    schema::{roles, users},
    utils::{parse_uuid, validate_dto, Pagination},
};

// ========== Query Parameters ==========

#[derive(Debug, FromForm)]
pub struct EmployeeQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub role_id: Option<String>,
    pub is_active: Option<bool>,
    pub search: Option<String>,
}

#[get("/api/v1/employees?<query..>")]
pub async fn list_employees(
    mut db: DbConnection,
    query: EmployeeQuery,
    admin: AdminGuard,
) -> Result<Json<PaginatedResponse<UserDto>>, ApiError> {
    let pagination = Pagination::new(query.page, query.per_page);

    log::info!(
        "Admin {} listing employees: page={}, per_page={}",
        admin.0.user_id,
        pagination.page,
        pagination.per_page
    );

    // Validate role_id UUID if provided
    let role_filter = if let Some(role_id_str) = query.role_id {
        Some(parse_uuid(&role_id_str, "role")?)
    } else {
        None
    };

    // Build quey with filters (same pattern as products.rs)
    let build_filtered_quey = || {
        let mut query_builder = users::table.inner_join(roles::table).into_boxed();

        if let Some(role_id) = role_filter {
            query_builder = query_builder.filter(users::role_id.eq(role_id));
        }

        if let Some(is_active) = query.is_active {
            query_builder = query_builder.filter(users::is_active.eq(is_active));
        }

        if let Some(search_term) = &query.search {
            let pattern = format!("%{}%", search_term);
            query_builder = query_builder.filter(
                users::name
                    .ilike(pattern.clone())
                    .or(users::email.ilike(pattern)),
            );
        }

        query_builder.order(users::created_at.desc())
    };

    let total_count = build_filtered_quey()
        .count()
        .get_result::<i64>(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to count employees: {}", e);
            ApiError::InternalError("Failed to retrieve employees".to_string())
        })?;

    let results = build_filtered_quey()
        .select((User::as_select(), Role::as_select()))
        .limit(pagination.per_page)
        .offset(pagination.offset)
        .load::<(User, Role)>(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to load employees: {}", e);
            ApiError::InternalError("Failed to retrieve employees".to_string())
        })?;

    let emplyee_dtos: Vec<UserDto> = results
        .into_iter()
        .map(|(user, role)| user.into_dto(role))
        .collect();

    let total_pages = pagination.total_pages(total_count);

    log::info!(
        "Retrieved {} employees (page {}/{})",
        emplyee_dtos.len(),
        pagination.page,
        total_pages
    );

    Ok(Json(PaginatedResponse {
        data: emplyee_dtos,
        page: pagination.page,
        per_page: pagination.per_page,
        total_count,
        total_pages,
    }))
}

// ========== Get Single Employee ==========

#[get("/api/v1/employees/<id>")]
pub async fn get_employee(
    mut db: DbConnection,
    id: String,
    admin: AdminGuard,
) -> Result<Json<UserDto>, ApiError> {
    let employee_id = parse_uuid(&id, "employee")?;

    log::info!(
        "Admin {} fetching employee: {}",
        admin.0.user_id,
        employee_id
    );

    let result = users::table
        .inner_join(roles::table)
        .filter(users::id.eq(employee_id))
        .select((User::as_select(), Role::as_select()))
        .first::<(User, Role)>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error fetching employee {}: {}", employee_id, e);
            ApiError::InternalError("Failed to retrieve employee".to_string())
        })?;

    match result {
        Some((user, role)) => {
            log::info!("Employee found: {}", user.name);
            Ok(Json(user.into_dto(role)))
        }
        None => {
            log::warn!("Employee {} not found", employee_id);
            Err(ApiError::NotFound("Employee not found".to_string()))
        }
    }
}

// ========== Create Employee (Admin Only) ==========

#[post("/api/v1/employees", data = "<request>")]
pub async fn create_employee(
    mut db: DbConnection,
    admin: AdminGuard,
    request: Json<CreateEmployeeRequest>,
) -> Result<(Status, Json<UserDto>), ApiError> {
    // Validation using validator crate
    validate_dto(&*request)?;

    let req = request.into_inner();

    log::info!(
        "Admin {} creating employee: {} ({})",
        admin.0.user_id,
        req.name,
        req.email
    );

    // Check email uniqueness
    let existing_user = users::table
        .filter(users::email.eq(&req.email))
        .select(users::id)
        .first::<Uuid>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error checking email: {}", e);
            ApiError::InternalError("Failed to validate email".to_string())
        })?;

    if existing_user.is_some() {
        log::warn!("Email {} already exists", req.email);
        return Err(ApiError::ValidationError(
            "Email already in use".to_string(),
        ));
    }

    // Verify role exists
    let role_exists = roles::table
        .filter(roles::id.eq(req.role_id))
        .select(roles::id)
        .first::<Uuid>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error checking role: {}", e);
            ApiError::InternalError("Failed to validate role".to_string())
        })?;

    if role_exists.is_none() {
        log::warn!("Role {} not found", req.role_id);
        return Err(ApiError::ValidationError("Role does not exist".to_string()));
    }

    // Generate temporary password (16 alphanumeric chars)
    use rand::Rng;
    let temp_password: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    // Hash password
    let password_hash = hash_password(&temp_password).map_err(|e| {
        log::error!("Failed to hash password: {}", e);
        ApiError::InternalError("Failed to create employee".to_string())
    })?;

    // Insert employee
    let new_user = NewUser {
        email: req.email,
        password_hash,
        name: req.name,
        role_id: req.role_id,
    };

    let inserted: User = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to insert employee: {}", e);
            ApiError::InternalError("Failed to create employee".to_string())
        })?;

    // Generate verification token (32 alphanumeric chars)
    let verification_token: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    // Set password reset token (7-day expiry)
    use chrono::{Duration, Utc};
    let token_expiry = Utc::now() + Duration::days(7);

    diesel::update(users::table.filter(users::id.eq(inserted.id)))
        .set((
            users::password_reset_token.eq(Some(verification_token.clone())),
            users::password_reset_expires_at.eq(Some(token_expiry)),
        ))
        .execute(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to set verification token: {}", e);
            ApiError::InternalError("Employee created but failed to generate token".to_string())
        })?;

    // Fetch with role for DTO
    let (user, role) = users::table
        .inner_join(roles::table)
        .filter(users::id.eq(inserted.id))
        .select((User::as_select(), Role::as_select()))
        .first::<(User, Role)>(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to fetch created employee: {}", e);
            ApiError::InternalError("Employee created but failed to retrieve".to_string())
        })?;

    log::info!("Employee {} created: {}", user.id, user.email);
    // TODO: Send verification email
    // TODO: Send email with verification_token

    Ok((Status::Created, Json(user.into_dto(role))))
}

// ========== Update Employee (Admin Only) ==========

#[put("/api/v1/employees/<id>", data = "<request>")]
pub async fn update_employee(
    mut db: DbConnection,
    admin: AdminGuard,
    id: String,
    request: Json<UpdateEmployeeRequest>,
) -> Result<Json<UserDto>, ApiError> {
    let employee_id = parse_uuid(&id, "employee")?;

    // Validation
    validate_dto(&*request)?;

    let req = request.into_inner();

    log::info!(
        "Admin {} updating employee: {}",
        admin.0.user_id,
        employee_id
    );

    // Check employee exists
    let exists = users::table
        .filter(users::id.eq(employee_id))
        .select(users::id)
        .first::<Uuid>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error checking employee: {}", e);
            ApiError::InternalError("Failed to check employee".to_string())
        })?;

    if exists.is_none() {
        log::warn!("Employee {} not found", employee_id);
        return Err(ApiError::NotFound("Employee not found".to_string()));
    }

    // Verify new role if provided
    if let Some(new_role_id) = req.role_id {
        let role_exists = roles::table
            .filter(roles::id.eq(new_role_id))
            .select(roles::id)
            .first::<Uuid>(&mut db.0)
            .optional()
            .map_err(|e| {
                log::error!("Database error checking role: {}", e);
                ApiError::InternalError("Failed to validate role".to_string())
            })?;

        if role_exists.is_none() {
            log::warn!("Role {} not found", new_role_id);
            return Err(ApiError::ValidationError("Role does not exist".to_string()));
        }
    }

    // Build update struct
    let update_data = UpdateUser {
        name: req.name,
        is_active: req.is_active,
        email_verified: None,
        email_verified_at: None,
        password_reset_token: None,
        password_reset_expires_at: None,
        last_login_at: None,
    };

    // Update user fields
    diesel::update(users::table.filter(users::id.eq(employee_id)))
        .set(&update_data)
        .execute(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to update employee {}: {}", employee_id, e);
            ApiError::InternalError("Failed to update employee".to_string())
        })?;

    // Update role separately (not in UpdateUser struct)
    if let Some(new_role_id) = req.role_id {
        diesel::update(users::table.filter(users::id.eq(employee_id)))
            .set(users::role_id.eq(new_role_id))
            .execute(&mut db.0)
            .map_err(|e| {
                log::error!("Failed to update employee role: {}", e);
                ApiError::InternalError("Failed to update employee role".to_string())
            })?;
    }

    // Fetch updated employee
    let (user, role) = users::table
        .inner_join(roles::table)
        .filter(users::id.eq(employee_id))
        .select((User::as_select(), Role::as_select()))
        .first::<(User, Role)>(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to fetch updated employee: {}", e);
            ApiError::InternalError("Employee updated but failed to retrieve".to_string())
        })?;

    log::info!("Employee {} updated", employee_id);

    Ok(Json(user.into_dto(role)))
}

// ========== Delete Employee - Soft Delete (Admin Only) ==========

#[delete("/api/v1/employees/<id>")]
pub async fn delete_employee(
    mut db: DbConnection,
    admin: AdminGuard,
    id: String,
) -> Result<Json<Value>, ApiError> {
    let employee_id = parse_uuid(&id, "employee")?;

    log::info!(
        "Admin {} deleting employee: {}",
        admin.0.user_id,
        employee_id
    );

    // Check exists and is active
    let employee = users::table
        .filter(users::id.eq(employee_id))
        .filter(users::is_active.eq(true))
        .select(User::as_select())
        .first::<User>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error checking employee: {}", e);
            ApiError::InternalError("Failed to check employee".to_string())
        })?;

    if employee.is_none() {
        log::warn!("Employee {} not found or already deleted", employee_id);
        return Err(ApiError::NotFound("Employee not found".to_string()));
    }

    // Soft delete
    diesel::update(users::table.filter(users::id.eq(employee_id)))
        .set(users::is_active.eq(false))
        .execute(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to delete employee {}: {}", employee_id, e);
            ApiError::InternalError("Failed to delete employee".to_string())
        })?;

    // Revoke all refresh tokens
    use crate::schema::refresh_tokens;
    if let Err(e) = diesel::update(
        refresh_tokens::table
            .filter(refresh_tokens::user_id.eq(employee_id))
            .filter(refresh_tokens::revoked_at.is_null()),
    )
    .set(refresh_tokens::revoked_at.eq(Some(chrono::Utc::now())))
    .execute(&mut db.0)
    {
        log::warn!(
            "Failed to revoke tokens for employee {}: {}",
            employee_id,
            e
        );
    }

    log::info!("Employee {} deactivated", employee_id);

    Ok(Json(serde_json::json!({
        "message": "Employee deactivated successfully",
        "employee_id": employee_id
    })))
}
