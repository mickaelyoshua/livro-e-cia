// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;

    categories (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        description -> Nullable<Text>,
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

    refresh_tokens (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        token_hash -> Varchar,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        revoked_at -> Nullable<Timestamptz>,
        last_used_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    roles (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password_hash -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        role_id -> Uuid,
        is_active -> Bool,
        email_verified -> Bool,
        email_verified_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        password_reset_token -> Nullable<Varchar>,
        password_reset_expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_login_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(products -> categories (category_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(users -> roles (role_id));

diesel::allow_tables_to_appear_in_same_query!(categories, products, refresh_tokens, roles, users,);
