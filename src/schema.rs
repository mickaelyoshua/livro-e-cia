// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "payment_method"))]
    pub struct PaymentMethod;
}

diesel::table! {
    use diesel::sql_types::*;

    categories (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    employees (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 50]
        role -> Varchar,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    products (id) {
        id -> Uuid,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        author -> Varchar,
        price -> Numeric,
        stock_quantity -> Int4,
        #[max_length = 255]
        publisher -> Nullable<Varchar>,
        publication_date -> Nullable<Date>,
        category_id -> Uuid,
        description -> Nullable<Text>,
        #[max_length = 500]
        cover_image_url -> Nullable<Varchar>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    refresh_token_families (id) {
        id -> Uuid,
        employee_id -> Uuid,
        current_jti -> Uuid,
        is_revoked -> Bool,
        created_at -> Timestamptz,
        last_used_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    roles (name) {
        #[max_length = 50]
        name -> Varchar,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    sale_items (sale_id, product_id) {
        sale_id -> Uuid,
        product_id -> Uuid,
        quantity -> Int4,
        unit_price -> Numeric,
        subtotal -> Numeric,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PaymentMethod;

    sales (id) {
        id -> Uuid,
        seller_id -> Uuid,
        subtotal -> Numeric,
        discount -> Numeric,
        total -> Numeric,
        payment_method -> PaymentMethod,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(employees -> roles (role));
diesel::joinable!(products -> categories (category_id));
diesel::joinable!(refresh_token_families -> employees (employee_id));
diesel::joinable!(sale_items -> products (product_id));
diesel::joinable!(sale_items -> sales (sale_id));
diesel::joinable!(sales -> employees (seller_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    employees,
    products,
    refresh_token_families,
    roles,
    sale_items,
    sales,
);
