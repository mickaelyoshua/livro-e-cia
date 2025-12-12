// Integration tests for product endpoints
//
// Tests: list, get, create, update, delete products

mod common;

use common::{
    bearer_header, create_default_category, create_employee_user, create_inactive_product,
    create_product, create_verified_admin, TestApp,
};
use rocket::http::{ContentType, Status};
use rust_decimal::Decimal;
use serial_test::serial;
use shared::{CreateProductRequest, PaginatedResponse, ProductDto, UpdateProductRequest};
use uuid::Uuid;

// ============================================
// GET /api/v1/products (List)
// ============================================

#[test]
#[serial]
fn list_products_requires_auth() {
    let app = TestApp::new();
    app.reset_database();

    let response = app.client.get("/api/v1/products").dispatch();

    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
#[serial]
fn list_products_returns_paginated_response() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let _product1 = create_product(&mut app.db_conn(), category.id);
    let _product2 = create_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(user.id, "admin");

    let response = app
        .client
        .get("/api/v1/products")
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let paginated: PaginatedResponse<ProductDto> =
        response.into_json().expect("Should parse paginated response");

    assert_eq!(paginated.data.len(), 2);
    assert_eq!(paginated.page, 1);
    assert!(paginated.per_page > 0);
    assert_eq!(paginated.total_count, 2);
}

#[test]
#[serial]
fn list_products_excludes_inactive() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let _active = create_product(&mut app.db_conn(), category.id);
    let _inactive = create_inactive_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(user.id, "admin");

    let response = app
        .client
        .get("/api/v1/products")
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let paginated: PaginatedResponse<ProductDto> = response.into_json().unwrap();
    // Only the active product should be returned
    assert_eq!(paginated.data.len(), 1);
    assert!(paginated.data[0].is_active);
}

#[test]
#[serial]
fn list_products_pagination_works() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());

    // Create 5 products
    for _ in 0..5 {
        create_product(&mut app.db_conn(), category.id);
    }

    let token = app.access_token_for(user.id, "admin");

    // Request page 1 with 2 items per page
    let response = app
        .client
        .get("/api/v1/products?page=1&per_page=2")
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let paginated: PaginatedResponse<ProductDto> = response.into_json().unwrap();
    assert_eq!(paginated.data.len(), 2);
    assert_eq!(paginated.page, 1);
    assert_eq!(paginated.per_page, 2);
    assert_eq!(paginated.total_count, 5);
    assert_eq!(paginated.total_pages, 3);
}

#[test]
#[serial]
fn list_products_filters_by_category() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let category1 = create_default_category(&mut app.db_conn());
    let category2 = create_default_category(&mut app.db_conn());

    let _product1 = create_product(&mut app.db_conn(), category1.id);
    let _product2 = create_product(&mut app.db_conn(), category2.id);

    let token = app.access_token_for(user.id, "admin");

    let response = app
        .client
        .get(format!("/api/v1/products?category_id={}", category1.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let paginated: PaginatedResponse<ProductDto> = response.into_json().unwrap();
    assert_eq!(paginated.data.len(), 1);
    assert_eq!(paginated.data[0].category.id, category1.id);
}

// ============================================
// GET /api/v1/products/<id> (Get Single)
// ============================================

#[test]
#[serial]
fn get_product_returns_product() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let product = create_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(user.id, "admin");

    let response = app
        .client
        .get(format!("/api/v1/products/{}", product.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let dto: ProductDto = response.into_json().expect("Should parse ProductDto");
    assert_eq!(dto.id, product.id);
    assert_eq!(dto.title, product.title);
}

#[test]
#[serial]
fn get_product_not_found() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(user.id, "admin");

    let response = app
        .client
        .get(format!("/api/v1/products/{}", Uuid::new_v4()))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
#[serial]
fn get_product_inactive_returns_404() {
    let app = TestApp::new();
    app.reset_database();

    let user = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let inactive = create_inactive_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(user.id, "admin");

    let response = app
        .client
        .get(format!("/api/v1/products/{}", inactive.id))
        .header(bearer_header(&token))
        .dispatch();

    // Soft-deleted products should appear as not found
    assert_eq!(response.status(), Status::NotFound);
}

// ============================================
// POST /api/v1/products (Create - Admin Only)
// ============================================

#[test]
#[serial]
fn create_product_requires_admin() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());

    let token = app.access_token_for(employee.id, "employee");

    let create_request = CreateProductRequest {
        title: "New Book".to_string(),
        author: "Test Author".to_string(),
        price: Decimal::new(1999, 2),
        stock_quantity: 5,
        publisher: None,
        publication_date: None,
        category_id: category.id,
        description: None,
        cover_image_url: None,
    };

    let response = app
        .client
        .post("/api/v1/products")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
#[serial]
fn create_product_as_admin_succeeds() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());

    let token = app.access_token_for(admin.id, "admin");

    let create_request = CreateProductRequest {
        title: "New Book".to_string(),
        author: "Test Author".to_string(),
        price: Decimal::new(1999, 2),
        stock_quantity: 5,
        publisher: Some("Test Publisher".to_string()),
        publication_date: None,
        category_id: category.id,
        description: Some("A great book".to_string()),
        cover_image_url: None,
    };

    let response = app
        .client
        .post("/api/v1/products")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Created);

    let product: ProductDto = response.into_json().expect("Should parse ProductDto");
    assert_eq!(product.title, "New Book");
    assert_eq!(product.author, "Test Author");
    assert!(product.is_active);
}

#[test]
#[serial]
fn create_product_invalid_category_fails() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    let create_request = CreateProductRequest {
        title: "New Book".to_string(),
        author: "Test Author".to_string(),
        price: Decimal::new(1999, 2),
        stock_quantity: 5,
        publisher: None,
        publication_date: None,
        category_id: Uuid::new_v4(), // Non-existent category
        description: None,
        cover_image_url: None,
    };

    let response = app
        .client
        .post("/api/v1/products")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
#[serial]
fn create_product_validation_errors() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    // Empty title
    let create_request = serde_json::json!({
        "title": "",
        "author": "Author",
        "price": "19.99",
        "stock_quantity": 5,
        "category_id": category.id
    });

    let response = app
        .client
        .post("/api/v1/products")
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&create_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

// ============================================
// PUT /api/v1/products/<id> (Update - Admin Only)
// ============================================

#[test]
#[serial]
fn update_product_requires_admin() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let product = create_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(employee.id, "employee");

    let update_request = UpdateProductRequest {
        title: Some("Updated Title".to_string()),
        author: None,
        price: None,
        stock_quantity: None,
        publisher: None,
        publication_date: None,
        category_id: None,
        description: None,
        cover_image_url: None,
        is_active: None,
    };

    let response = app
        .client
        .put(format!("/api/v1/products/{}", product.id))
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&update_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
#[serial]
fn update_product_as_admin_succeeds() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let product = create_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(admin.id, "admin");

    let update_request = UpdateProductRequest {
        title: Some("Updated Title".to_string()),
        author: None,
        price: Some(Decimal::new(3999, 2)),
        stock_quantity: None,
        publisher: None,
        publication_date: None,
        category_id: None,
        description: None,
        cover_image_url: None,
        is_active: None,
    };

    let response = app
        .client
        .put(format!("/api/v1/products/{}", product.id))
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&update_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let updated: ProductDto = response.into_json().expect("Should parse ProductDto");
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.price, Decimal::new(3999, 2));
}

#[test]
#[serial]
fn update_product_not_found() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    let update_request = UpdateProductRequest {
        title: Some("Updated Title".to_string()),
        author: None,
        price: None,
        stock_quantity: None,
        publisher: None,
        publication_date: None,
        category_id: None,
        description: None,
        cover_image_url: None,
        is_active: None,
    };

    let response = app
        .client
        .put(format!("/api/v1/products/{}", Uuid::new_v4()))
        .header(bearer_header(&token))
        .header(ContentType::JSON)
        .body(serde_json::to_string(&update_request).unwrap())
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

// ============================================
// DELETE /api/v1/products/<id> (Soft Delete - Admin Only)
// ============================================

#[test]
#[serial]
fn delete_product_requires_admin() {
    let app = TestApp::new();
    app.reset_database();

    let employee = create_employee_user(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let product = create_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(employee.id, "employee");

    let response = app
        .client
        .delete(format!("/api/v1/products/{}", product.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
}

#[test]
#[serial]
fn delete_product_soft_deletes() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let category = create_default_category(&mut app.db_conn());
    let product = create_product(&mut app.db_conn(), category.id);

    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .delete(format!("/api/v1/products/{}", product.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Verify product is soft-deleted (not found in normal queries)
    let get_response = app
        .client
        .get(format!("/api/v1/products/{}", product.id))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(get_response.status(), Status::NotFound);

    // But still exists in DB with is_active = false
    use backend::schema::products;
    use diesel::prelude::*;

    let deleted_product: backend::models::Product = products::table
        .find(product.id)
        .first(&mut app.db_conn())
        .expect("Product should still exist in DB");

    assert!(!deleted_product.is_active);
}

#[test]
#[serial]
fn delete_product_not_found() {
    let app = TestApp::new();
    app.reset_database();

    let admin = create_verified_admin(&mut app.db_conn());
    let token = app.access_token_for(admin.id, "admin");

    let response = app
        .client
        .delete(format!("/api/v1/products/{}", Uuid::new_v4()))
        .header(bearer_header(&token))
        .dispatch();

    assert_eq!(response.status(), Status::NotFound);
}
