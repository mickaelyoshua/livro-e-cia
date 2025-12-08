use diesel::prelude::*;
use rocket::{delete, get, http::Status, post, put, serde::json::Json, FromForm};
use serde_json::Value;
use shared::{CreateProductRequest, PaginatedResponse, ProductDto, UpdateProductRequest};
use uuid::Uuid;

use crate::{
    auth::{AdminGuard, AuthUser},
    db::DbConnection,
    error::ApiError,
    models::{Category, NewProduct, Product, UpdateProduct},
    schema::{categories, products},
    utils::validate_dto,
};

// ========== Query Parameters ==========

#[derive(Debug, FromForm)]
pub struct ProductQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub category_id: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
}

// ========== List Products (Paginated) ==========

/// GET /api/v1/products?page&per_page&category_id&search&sort
#[get("/api/v1/products?<query..>")]
pub async fn list_products(
    mut db: DbConnection,
    query: ProductQuery,
    _user: AuthUser,
) -> Result<Json<PaginatedResponse<ProductDto>>, ApiError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    log::info!("Listing products: page={}, per_page={}", page, per_page);

    // Validate category_id
    let category_filter = if let Some(cat_id_str) = query.category_id {
        Some(Uuid::parse_str(&cat_id_str).map_err(|e| {
            log::warn!("Invalid category_id UUID: {}", e);
            ApiError::ValidationError("Invalid category_id format".to_string())
        })?)
    } else {
        None
    };

    // Validate sort field
    let sort_field = query.sort.as_deref().unwrap_or("-created_at");
    // the '-' indicates descendant

    if ![
        "title",
        "-title",
        "author",
        "-author",
        "price",
        "-price",
        "created_at",
        "-created_at",
    ]
    .contains(&sort_field)
    {
        log::warn!("Invalid sort field: {}", sort_field);
        return Err(ApiError::ValidationError(
            "Invalid sort field. Use: title, author, price, created_at (prefix - for desc)"
                .to_string(),
        ));
    }

    // Making a builder because i can not reuse the same query twice
    let build_filtered_query = || {
        // Dynamic query with boxed for conditional filters
        let mut query_builder = products::table
            .inner_join(categories::table)
            .filter(products::is_active.eq(true))
            // Each filter on diesel creates a different type, boxed allows the same type always and to
            // be able to make those separations in variables, reasign query to different types
            .into_boxed();

        // Filter by category (already validated)
        if let Some(cat_id) = category_filter {
            query_builder = query_builder.filter(products::category_id.eq(cat_id));
        }

        // Search by title or author (case-insensitive)
        if let Some(search_term) = &query.search {
            let pattern = format!("%{}%", search_term);

            query_builder = query_builder.filter(
                products::title
                    .ilike(pattern.clone())
                    .or(products::author.ilike(pattern)),
            );
        }

        // Sort (already validated)
        match sort_field {
            "title" => query_builder.order(products::title.asc()),
            "-title" => query_builder.order(products::title.desc()),
            "author" => query_builder.order(products::author.asc()),
            "-author" => query_builder.order(products::author.desc()),
            "price" => query_builder.order(products::price.asc()),
            "-price" => query_builder.order(products::price.desc()),
            "created_at" => query_builder.order(products::created_at.asc()),
            "-created_at" => query_builder.order(products::created_at.desc()),
            _ => query_builder,
        }
    };

    let total_count = build_filtered_query()
        .count()
        .get_result::<i64>(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to count products: {}", e);
            ApiError::InternalError("Failed to retrieve products".to_string())
        })?;

    // Get paginated result
    let results = build_filtered_query()
        .select((Product::as_select(), Category::as_select()))
        .limit(per_page)
        .offset(offset)
        .load::<(Product, Category)>(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to load products: {}", e);
            ApiError::InternalError("Failed to retrieve products".to_string())
        })?;

    let product_dtos: Vec<ProductDto> = results
        .into_iter()
        .map(|(product, category)| product.into_dto(category))
        .collect();

    let total_pages = (total_count as f64 / per_page as f64).ceil() as i64;

    log::info!(
        "Retrieved {} products (page {}/{})",
        product_dtos.len(),
        page,
        total_pages
    );

    Ok(Json(PaginatedResponse {
        data: product_dtos,
        page,
        per_page,
        total_count,
        total_pages,
    }))
}

// ========== Get Single Product ==========

#[get("/api/v1/products/<id>")]
pub async fn get_product(
    mut db: DbConnection,
    id: String,
    _user: AuthUser,
) -> Result<Json<ProductDto>, ApiError> {
    let product_id = Uuid::parse_str(&id).map_err(|e| {
        log::warn!("Invalid product UUID: {}", e);
        ApiError::ValidationError("Invalid Product ID format".to_string())
    })?;

    log::info!("Fetching product: {}", product_id);

    let result = products::table
        .inner_join(categories::table)
        .filter(products::id.eq(product_id))
        .filter(products::is_active.eq(true))
        .select((Product::as_select(), Category::as_select()))
        .first::<(Product, Category)>(&mut db.0)
        // instead of returning a Result<T, Error>, optional returns a Result<Option<T>, Error>,
        // this allows to check if the query had no error (everything ok) but does not found any
        // data (empty result)
        .optional()
        .map_err(|e| {
            log::error!("Database error fetching product {}: {}", product_id, e);
            ApiError::InternalError("Failed to retrieve product".to_string())
        })?;

    match result {
        Some((product, category)) => {
            log::info!("Product found: {}", product.title);
            Ok(Json(product.into_dto(category)))
        }
        None => {
            log::warn!("Product {} not found", product_id);
            Err(ApiError::NotFound("Product not found".to_string()))
        }
    }
}

// ========== Create Product (Admin Only) ==========

#[post("/api/v1/products", data = "<request>")]
pub async fn create_product(
    mut db: DbConnection,
    admin: AdminGuard,
    request: Json<CreateProductRequest>,
) -> Result<(Status, Json<ProductDto>), ApiError> {
    // Fields validation
    validate_dto(&*request)?;

    let req = request.into_inner();

    log::info!("Admin {} creating product: {}", admin.0.user_id, req.title);

    // Verify category exists
    let category_exists = categories::table
        .filter(categories::id.eq(req.category_id))
        .select(categories::id)
        .first::<Uuid>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database erro checking category: {}", e);
            ApiError::InternalError("Failed to validate category".to_string())
        })?;

    if category_exists.is_none() {
        log::warn!("Category {} not found", req.category_id);
        return Err(ApiError::ValidationError(
            "Category does not exist".to_string(),
        ));
    }

    let new_product = NewProduct {
        title: req.title,
        author: req.author,
        price: req.price,
        stock_quantity: req.stock_quantity,
        publisher: req.publisher,
        publication_date: req.publication_date,
        category_id: req.category_id,
        description: req.description,
        cover_image_url: req.cover_image_url,
    };

    let inserted: Product = diesel::insert_into(products::table)
        .values(&new_product)
        .returning(Product::as_returning())
        .get_result(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to insert product: {}", e);
            ApiError::InternalError("Failed to create product".to_string())
        })?;

    // Fetch with category
    let (product, category) = products::table
        .inner_join(categories::table)
        .filter(products::id.eq(inserted.id))
        .select((Product::as_select(), Category::as_select()))
        .first::<(Product, Category)>(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to fetch created product: {}", e);
            ApiError::InternalError("Product created but failed to retrieve".to_string())
        })?;

    log::info!("Product {} created: {}", product.id, product.title);

    Ok((Status::Created, Json(product.into_dto(category))))
}

// ========== Update Product (Admin Only) ==========

#[put("/api/v1/products/<id>", data = "<request>")]
pub async fn update_product(
    mut db: DbConnection,
    admin: AdminGuard,
    id: String,
    request: Json<UpdateProductRequest>,
) -> Result<Json<ProductDto>, ApiError> {
    // Validate Uuid
    let product_id = Uuid::parse_str(&id).map_err(|e| {
        log::warn!("Invalid product UUID: {}", e);
        ApiError::ValidationError("Invalid product ID format".to_string())
    })?;

    // Validate fields
    validate_dto(&*request)?;

    let req = request.into_inner();

    log::info!("Admin {} updating product: {}", admin.0.user_id, product_id);

    let exists = products::table
        .filter(products::id.eq(product_id))
        .filter(products::is_active.eq(true))
        .select(products::id)
        .first::<Uuid>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error checking product: {}", e);
            ApiError::InternalError("Failed to check product".to_string())
        })?;

    if exists.is_none() {
        log::warn!("Product {} not found", product_id);
        return Err(ApiError::NotFound("Product not found".to_string()));
    }

    // Verify new category if provided
    if let Some(new_cat_id) = req.category_id {
        let cat_exists = categories::table
            .filter(categories::id.eq(new_cat_id))
            .select(categories::id)
            .first::<Uuid>(&mut db.0)
            .optional()
            .map_err(|e| {
                log::error!("Database error checking category: {}", e);
                ApiError::InternalError("Failed to validate category".to_string())
            })?;

        if cat_exists.is_none() {
            log::warn!("Category {} not found", new_cat_id);
            return Err(ApiError::ValidationError(
                "Category does not exist".to_string(),
            ));
        }
    }

    let update_data = UpdateProduct {
        title: req.title,
        author: req.author,
        price: req.price,
        stock_quantity: req.stock_quantity,
        publisher: req.publisher.map(Some),
        publication_date: req.publication_date.map(Some),
        category_id: req.category_id,
        description: req.description.map(Some),
        cover_image_url: req.cover_image_url.map(Some),
        is_active: req.is_active,
    };

    // Update
    diesel::update(products::table.filter(products::id.eq(product_id)))
        .set(&update_data)
        .execute(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to update product {}: {}", product_id, e);
            ApiError::InternalError("Failed to update product".to_string())
        })?;

    // Fetch updated product
    let (product, category) = products::table
        .inner_join(categories::table)
        .filter(products::id.eq(product_id))
        .select((Product::as_select(), Category::as_select()))
        .first::<(Product, Category)>(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to fetch updated product: {}", e);
            ApiError::InternalError("Product updated but failed to retrieve".to_string())
        })?;

    log::info!("Product {} updated", product_id);

    Ok(Json(product.into_dto(category)))
}

// ========== Delete Product - Soft Delete (Admin Only) ==========

#[delete("/api/v1/products/<id>")]
pub async fn delete_product(
    mut db: DbConnection,
    admin: AdminGuard,
    id: String,
) -> Result<Json<Value>, ApiError> {
    let product_id = Uuid::parse_str(&id).map_err(|e| {
        log::warn!("Invalid product UUID: {}", e);
        ApiError::ValidationError("Invalid product ID format".to_string())
    })?;

    log::info!("Admin {} deleting product: {}", admin.0.user_id, product_id);

    let product = products::table
        .filter(products::id.eq(product_id))
        .filter(products::is_active.eq(true))
        .select(Product::as_select())
        .first::<Product>(&mut db.0)
        .optional()
        .map_err(|e| {
            log::error!("Database error checking product: {}", e);
            ApiError::InternalError("Failed to check product".to_string())
        })?;

    if product.is_none() {
        log::error!("Product {} not found or already deleted", product_id);
        return Err(ApiError::NotFound("Product not found".to_string()));
    }

    // Soft delete
    diesel::update(products::table.filter(products::id.eq(product_id)))
        .set(products::is_active.eq(false))
        .execute(&mut db.0)
        .map_err(|e| {
            log::error!("Failed to delete product {}: {}", product_id, e);
            ApiError::InternalError("Failed to delete product".to_string())
        })?;

    log::info!("Product {} soft deleted", product_id);

    Ok(Json(serde_json::json!({
        "message": "Product deleted successfully",
        "product_id": product_id
    })))
}
