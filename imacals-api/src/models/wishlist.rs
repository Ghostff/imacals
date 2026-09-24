use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::product::CatalogProduct;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wishlist {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub customer_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishlistItem {
    pub id: Uuid,
    pub wishlist_id: Uuid,
    pub product_id: Uuid,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// Wishlist item with its product snapshot embedded — what the storefront renders as a card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishlistItemWithProduct {
    pub id: Uuid,
    pub wishlist_id: Uuid,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub product: CatalogProduct,
}

// Summary used in list views — wishlist fields flattened together with the current item count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishlistSummary {
    #[serde(flatten)]
    pub wishlist: Wishlist,
    pub item_count: i64,
}

// Detail used in show views — wishlist fields flattened together with the active items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WishlistDetail {
    #[serde(flatten)]
    pub wishlist: Wishlist,
    pub items: Vec<WishlistItemWithProduct>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWishlistSchema {
    // Optional: when omitted, the service resolves the customer from the acting user.
    pub customer_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255, message = "Wishlist name must be between 1 and 255 characters"))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateWishlistSchema {
    #[validate(length(min = 1, max = 255, message = "Wishlist name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddWishlistItemSchema {
    pub product_id: Uuid,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_wishlist_schema_requires_name() {
        let result: Result<CreateWishlistSchema, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }

    #[test]
    fn create_wishlist_schema_accepts_valid_payload() {
        let json = r#"{"name": "Reorder list", "description": "For next visit"}"#;
        let schema: CreateWishlistSchema = serde_json::from_str(json).unwrap();
        assert!(schema.validate().is_ok());
    }

    #[test]
    fn add_item_requires_product_id() {
        let result: Result<AddWishlistItemSchema, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }
}
