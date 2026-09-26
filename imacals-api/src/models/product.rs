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
    // Promotional slash price in integer kobo. Must be less than unit_price_kobo when present.
    pub discount_price_kobo: Option<i64>,
    // Wholesale lines often cannot be bought as singles.
    pub min_order_quantity: i32,
    pub in_stock: bool,
    // Whether this product is exempt from Nigerian VAT (e.g. raw unprocessed foodstuff).
    pub is_tax_exempt: bool,
    // Statutory Nigerian VAT rate in basis points (750 = 7.50%).
    pub tax_rate_basis_points: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Product {
    // The price to actually charge: promotional discount price if present, else regular unit price.
    pub fn effective_price_kobo(&self) -> i64 {
        self.discount_price_kobo.unwrap_or(self.unit_price_kobo)
    }

    // Percentage discount off unit price rounded to nearest whole percent.
    pub fn discount_percent(&self) -> Option<i32> {
        self.discount_price_kobo.map(|discount| {
            if self.unit_price_kobo > 0 {
                let diff = self.unit_price_kobo - discount;
                ((diff as f64 / self.unit_price_kobo as f64) * 100.0).round() as i32
            } else {
                0
            }
        })
    }

    // Line tax in integer kobo: 0 if tax exempt, else (effective_price * quantity * rate + 5000) / 10000.
    pub fn line_tax_kobo(&self, quantity: i32) -> i64 {
        if self.is_tax_exempt || self.tax_rate_basis_points <= 0 || quantity <= 0 {
            0
        } else {
            let line_total = self.effective_price_kobo() * (quantity as i64);
            (line_total * (self.tax_rate_basis_points as i64) + 5000) / 10000
        }
    }
}

// Image attachment representation for a product.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductImageDto {
    pub id: Uuid,
    pub url: String,
    pub is_default: bool,
    pub name: String,
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
    pub discount_price_kobo: Option<i64>,
    pub discount_percent: Option<i32>,
    pub min_order_quantity: i32,
    pub in_stock: bool,
    pub is_tax_exempt: bool,
    pub tax_rate_basis_points: i32,
    pub image_url: Option<String>,
    #[serde(default)]
    pub images: Vec<String>,
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
    pub discount_price_kobo: Option<i64>,
    pub discount_percent: Option<i32>,
    pub min_order_quantity: i32,
    pub in_stock: bool,
    pub is_tax_exempt: bool,
    pub tax_rate_basis_points: i32,
    pub image_url: Option<String>,
    #[serde(default)]
    pub images: Vec<ProductImageDto>,
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
    pub discount_price_kobo: Option<i64>,
    pub min_order_quantity: Option<i32>,
    pub in_stock: Option<bool>,
    pub is_tax_exempt: Option<bool>,
    pub tax_rate_basis_points: Option<i32>,
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
    pub discount_price_kobo: Option<Option<i64>>,
    pub min_order_quantity: Option<i32>,
    pub in_stock: Option<bool>,
    pub is_tax_exempt: Option<bool>,
    pub tax_rate_basis_points: Option<i32>,
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
    fn create_product_schema_accepts_valid_payload_with_discount() {
        let json = serde_json::json!({
            "category_id": "00000000-0000-0000-0000-000000000001",
            "name": "Long Grain Rice — 50kg Bag",
            "slug": "rice-50kg",
            "description": "Parboiled long grain rice",
            "unit": "bag (50kg)",
            "unit_price_kobo": 8950000,
            "discount_price_kobo": 8200000,
            "min_order_quantity": 5,
            "in_stock": true
        });

        let schema: Result<CreateProductSchema, _> = serde_json::from_value(json);
        assert!(schema.is_ok());
        let valid = schema.unwrap();
        assert!(valid.validate().is_ok());
        assert_eq!(valid.unit_price_kobo, 8950000);
        assert_eq!(valid.discount_price_kobo, Some(8200000));
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

    #[test]
    fn product_effective_price_and_discount_percent() {
        let p = Product {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            domain_id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            created_by: Uuid::new_v4(),
            name: "Long Grain Rice".into(),
            slug: "rice".into(),
            description: None,
            unit: "bag".into(),
            unit_price_kobo: 1000000, // ₦10,000
            discount_price_kobo: Some(800000), // ₦8,000 (20% off)
            min_order_quantity: 1,
            in_stock: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        assert_eq!(p.effective_price_kobo(), 800000);
        assert_eq!(p.discount_percent(), Some(20));
    }
}
