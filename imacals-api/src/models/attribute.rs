use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// A key/value config entry attached to any owner entity via attributeable_type + attributeable_id.
// Designed for integration credentials (e.g. RETS server url, username, password).
// When is_encrypted = true, `value` holds the ciphertext — decrypt at the service layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub id: Uuid,
    pub created_by: Uuid,
    pub attributeable_type: String,
    pub attributeable_id: Uuid,
    pub name: String,
    pub value: Option<String>,
    // Serialised as "type" to match the DB column; "type" is reserved in Rust.
    #[serde(rename = "type")]
    pub attribute_type: String,
    pub is_encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// The form sent when creating an attribute.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateAttributeSchema {
    pub attributeable_type: String,
    pub attributeable_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Name must be 1–255 characters"))]
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "type")]
    #[validate(length(min = 1, max = 100, message = "Type must be 1–100 characters"))]
    pub attribute_type: String,
    pub is_encrypted: Option<bool>,
}

// PATCH semantics — every field optional.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAttributeSchema {
    #[validate(length(min = 1, max = 255, message = "Name must be 1–255 characters"))]
    pub name: Option<String>,
    pub value: Option<String>,
    #[serde(rename = "type")]
    #[validate(length(min = 1, max = 100, message = "Type must be 1–100 characters"))]
    pub attribute_type: Option<String>,
    pub is_encrypted: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_schema_requires_attributeable_type() {
        let result: Result<CreateAttributeSchema, _> = serde_json::from_str(
            r#"{"attributeable_id":"00000000-0000-0000-0000-000000000001","name":"url","type":"string"}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn create_schema_requires_attributeable_id() {
        let result: Result<CreateAttributeSchema, _> =
            serde_json::from_str(r#"{"attributeable_type":"integrations","name":"url","type":"string"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn create_schema_requires_name() {
        let result: Result<CreateAttributeSchema, _> = serde_json::from_str(
            r#"{"attributeable_type":"integrations","attributeable_id":"00000000-0000-0000-0000-000000000001","type":"string"}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn create_schema_requires_type() {
        let result: Result<CreateAttributeSchema, _> = serde_json::from_str(
            r#"{"attributeable_type":"integrations","attributeable_id":"00000000-0000-0000-0000-000000000001","name":"url"}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn create_schema_accepts_valid_payload() {
        let json = r#"{
            "attributeable_type": "integrations",
            "attributeable_id":   "00000000-0000-0000-0000-000000000001",
            "name":               "url",
            "value":              "https://rets.example.com",
            "type":               "url"
        }"#;
        let parsed: CreateAttributeSchema = serde_json::from_str(json).unwrap();
        assert!(parsed.is_encrypted.is_none());
    }

    #[test]
    fn create_schema_accepts_encrypted_password() {
        let json = r#"{
            "attributeable_type": "integrations",
            "attributeable_id":   "00000000-0000-0000-0000-000000000001",
            "name":               "password",
            "value":              "secret123",
            "type":               "password",
            "is_encrypted":       true
        }"#;
        let parsed: CreateAttributeSchema = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.is_encrypted, Some(true));
    }

    #[test]
    fn update_schema_all_fields_optional() {
        let parsed: UpdateAttributeSchema = serde_json::from_str("{}").unwrap();
        assert!(parsed.name.is_none());
        assert!(parsed.value.is_none());
        assert!(parsed.attribute_type.is_none());
    }
}
