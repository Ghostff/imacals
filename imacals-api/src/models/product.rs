use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// A sellable line in the catalogue. Stock status is in_stock; money is always integer kobo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub domain_id: Uuid,
    pub category_id: Uuid,
    pub created_by: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    // The unit one quantity buys — "carton", "bag (50kg)", "piece".
    pub unit: String,
    // Kobo, never naira: integer money is the only kind that survives arithmetic without rounding drift.
    pub unit_price_kobo: i64,
    // Wholesale lines often cannot be bought as singles.
    pub min_order_quantity: i32,
    pub in_stock: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// Product representation returned to the public customer storefront.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogProduct {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub category_slug: String,
    pub category_name: String,
    pub unit: String,
    pub unit_price_kobo: i64,
    pub min_order_quantity: i32,
    pub in_stock: bool,
    pub image_url: Option<String>,
}

// Product representation returned to the staff dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminProduct {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub domain_id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub category_slug: String,
    pub created_by: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub unit: String,
    pub unit_price_kobo: i64,
    pub min_order_quantity: i32,
    pub in_stock: bool,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Payload sent by the dashboard when creating a product.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductSchema {
    pub domain_id: Option<Uuid>,
    pub category_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 255, message = "Slug must be between 1 and 255 characters"))]
    pub slug: String,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 100, message = "Unit must be specified (e.g. carton, bag (50kg))"))]
    pub unit: String,
    #[validate(range(min = 1, message = "Price must be greater than zero kobo"))]
    pub unit_price_kobo: i64,
    pub min_order_quantity: Option<i32>,
    pub in_stock: Option<bool>,
}

// Payload sent by the dashboard when updating a product.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProductSchema {
    pub domain_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255, message = "Slug must be between 1 and 255 characters"))]
    pub slug: Option<String>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub unit_price_kobo: Option<i64>,
    pub min_order_quantity: Option<i32>,
    pub in_stock: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_product_schema_requires_name_unit_and_price() {
        let result: Result<CreateProductSchema, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }

    #[test]
    fn create_product_schema_accepts_valid_payload() {
        let json = serde_json::json!({
            "category_id": "00000000-0000-0000-0000-000000000001",
            "name": "Long Grain Rice — 50kg Bag",
            "slug": "rice-50kg",
            "description": "Parboiled long grain rice",
            "unit": "bag (50kg)",
            "unit_price_kobo": 8950000,
            "min_order_quantity": 5,
            "in_stock": true
        });

        let schema: Result<CreateProductSchema, _> = serde_json::from_value(json);
        assert!(schema.is_ok());
        let valid = schema.unwrap();
        assert!(valid.validate().is_ok());
        assert_eq!(valid.unit_price_kobo, 8950000);
    }

    #[test]
    fn zero_price_fails_validation() {
        let json = serde_json::json!({
            "category_id": "00000000-0000-0000-0000-000000000001",
            "name": "Free Sample",
            "slug": "sample",
            "unit": "piece",
            "unit_price_kobo": 0
        });

        let schema: CreateProductSchema = serde_json::from_value(json).unwrap();
        assert!(schema.validate().is_err());
    }
}
