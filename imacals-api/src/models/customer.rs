use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// A buyer. May exist without a users row — phone orders create one from a name and number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub full_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    // NULL for phone-only customers; populated for online customers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCustomerSchema {
    #[validate(length(min = 1, max = 255, message = "Full name must be between 1 and 255 characters"))]
    pub full_name: String,
    pub phone: Option<String>,
    #[validate(email(message = "Email must be a valid address"))]
    pub email: Option<String>,
    // Linking an existing user account is optional — phone-only customers have none.
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCustomerSchema {
    #[validate(length(min = 1, max = 255, message = "Full name must be between 1 and 255 characters"))]
    pub full_name: Option<String>,
    pub phone: Option<String>,
    #[validate(email(message = "Email must be a valid address"))]
    pub email: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_customer_schema_requires_full_name() {
        let result: Result<CreateCustomerSchema, _> = serde_json::from_str("{}");
        assert!(result.is_err());
    }

    #[test]
    fn create_customer_schema_accepts_phone_customer() {
        let json = r#"{"full_name": "Ada Eze", "phone": "08030000000"}"#;
        let schema: Result<CreateCustomerSchema, _> = serde_json::from_str(json);
        assert!(schema.is_ok());
        assert!(schema.unwrap().validate().is_ok());
    }

    #[test]
    fn update_customer_schema_rejects_invalid_email() {
        let json = r#"{"email": "not-an-email"}"#;
        let schema: UpdateCustomerSchema = serde_json::from_str(json).unwrap();
        assert!(schema.validate().is_err());
    }
}
