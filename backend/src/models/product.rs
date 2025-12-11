use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(crate::models::Category))]
#[diesel(table_name = crate::schema::products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Product {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub price: rust_decimal::Decimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Uuid,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::products)]
pub struct NewProduct {
    pub title: String,
    pub author: String,
    pub price: rust_decimal::Decimal,
    pub stock_quantity: i32,
    pub publisher: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub category_id: Uuid,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = crate::schema::products)]
pub struct UpdateProduct {
    pub title: Option<String>,
    pub author: Option<String>,
    pub price: Option<rust_decimal::Decimal>,
    pub stock_quantity: Option<i32>,
    pub publisher: Option<Option<String>>,
    pub publication_date: Option<Option<NaiveDate>>,
    pub category_id: Option<Uuid>,
    pub description: Option<Option<String>>,
    pub cover_image_url: Option<Option<String>>,
    pub is_active: Option<bool>,
}

impl Product {
    pub fn into_dto(self, category: crate::models::Category) -> shared::ProductDto {
        shared::ProductDto {
            id: self.id,
            title: self.title,
            author: self.author,
            price: self.price,
            stock_quantity: self.stock_quantity,
            publisher: self.publisher,
            publication_date: self.publication_date,
            category: category.into_dto(),
            description: self.description,
            cover_image_url: self.cover_image_url,
            is_active: self.is_active,
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Category;
    use chrono::{NaiveDate, Utc};
    use rust_decimal::Decimal;

    fn sample_category() -> Category {
        Category {
            id: Uuid::new_v4(),
            name: "Fiction".to_string(),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_product() -> Product {
        Product {
            id: Uuid::new_v4(),
            title: "The Great Gatsby".to_string(),
            author: "F. Scott Fitzgerald".to_string(),
            price: Decimal::new(2999, 2), // 29.99
            stock_quantity: 10,
            publisher: Some("Scribner".to_string()),
            publication_date: Some(NaiveDate::from_ymd_opt(1925, 4, 10).unwrap()),
            category_id: Uuid::new_v4(),
            description: Some("A classic novel".to_string()),
            cover_image_url: Some("https://example.com/cover.jpg".to_string()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn into_dto_maps_required_fields() {
        let product = sample_product();
        let id = product.id;
        let title = product.title.clone();
        let author = product.author.clone();
        let category = sample_category();
        let dto = product.into_dto(category);

        assert_eq!(dto.id, id);
        assert_eq!(dto.title, title);
        assert_eq!(dto.author, author);
        assert_eq!(dto.stock_quantity, 10);
        assert!(dto.is_active);
    }

    #[test]
    fn into_dto_preserves_decimal_precision() {
        let product = sample_product();
        let category = sample_category();
        let dto = product.into_dto(category);

        assert_eq!(dto.price, Decimal::new(2999, 2));
    }

    #[test]
    fn into_dto_embeds_category() {
        let product = sample_product();
        let category = sample_category();
        let category_name = category.name.clone();
        let dto = product.into_dto(category);

        assert_eq!(dto.category.name, category_name);
    }

    #[test]
    fn into_dto_maps_optional_fields() {
        let product = sample_product();
        let category = sample_category();
        let dto = product.into_dto(category);

        assert_eq!(dto.publisher, Some("Scribner".to_string()));
        assert!(dto.publication_date.is_some());
        assert!(dto.description.is_some());
        assert!(dto.cover_image_url.is_some());
    }

    #[test]
    fn into_dto_handles_all_none_optional_fields() {
        let mut product = sample_product();
        product.publisher = None;
        product.publication_date = None;
        product.description = None;
        product.cover_image_url = None;

        let category = sample_category();
        let dto = product.into_dto(category);

        assert!(dto.publisher.is_none());
        assert!(dto.publication_date.is_none());
        assert!(dto.description.is_none());
        assert!(dto.cover_image_url.is_none());
    }

    #[test]
    fn into_dto_omits_updated_at() {
        // ProductDto has created_at but NOT updated_at
        let product = sample_product();
        let category = sample_category();
        let _dto: shared::ProductDto = product.into_dto(category);
    }
}
