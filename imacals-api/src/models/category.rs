use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// A product category scoped to a geographic domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: Uuid,
    pub domain_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// Payload for creating a new category.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCategorySchema {
    pub domain_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 255, message = "Slug must be between 1 and 255 characters"))]
    pub slug: String,
    pub description: Option<String>,
}

// Payload for updating an existing category.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCategorySchema {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 255, message = "Slug must be between 1 and 255 characters"))]
    pub slug: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_category_schema_requires_name_and_slug() {
        let result: Result<CreateCategorySchema, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }

    #[test]
    fn create_category_schema_accepts_valid_payload() {
        let json = r#"{"name": "Foodstuff", "slug": "foodstuff", "description": "Staple food items"}"#;
        let schema: Result<CreateCategorySchema, _> = serde_json::from_str(json);
        assert!(schema.is_ok());
        let valid = schema.unwrap();
        assert!(valid.validate().is_ok());
        assert_eq!(valid.name, "Foodstuff");
        assert_eq!(valid.slug, "foodstuff");
    }
}
