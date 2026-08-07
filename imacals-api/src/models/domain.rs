use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Domains are location-scoped namespaces for reference data that varies by market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub country_id: Uuid,
    pub state_id: Option<Uuid>,
    pub city_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// The form a caller sends when creating or fully replacing a domain.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDomainSchema {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 255, message = "Slug must be between 1 and 255 characters"))]
    pub slug: String,
    pub country_id: Uuid,
    pub state_id: Option<Uuid>,
    pub city_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "id":         "00000000-0000-0000-0000-000000000001",
        "name":       "Default US",
        "slug":       "default-us",
        "country_id": "00000000-0000-0000-0000-000000000002",
        "state_id":   null,
        "city_id":    null,
        "created_at": "2025-01-01T00:00:00Z",
        "updated_at": "2025-01-01T00:00:00Z"
    }"#;

    #[test]
    fn deserializes_with_optional_fields_absent() {
        let d: Domain = serde_json::from_str(SAMPLE).unwrap();
        assert_eq!(d.name, "Default US");
        assert!(d.state_id.is_none());
        assert!(d.city_id.is_none());
        assert!(d.deleted_at.is_none());
    }

    // deleted_at must be omitted from JSON when null to keep the wire format clean.
    #[test]
    fn deleted_at_is_skipped_when_none() {
        let d: Domain = serde_json::from_str(SAMPLE).unwrap();
        let v = serde_json::to_value(&d).unwrap();
        assert!(v.get("deleted_at").is_none(), "deleted_at should be absent when None");
    }

    // A payload missing name must fail to deserialize.
    #[test]
    fn create_schema_requires_name() {
        let result: Result<CreateDomainSchema, _> = serde_json::from_str(
            r#"{"slug":"x","country_id":"00000000-0000-0000-0000-000000000001"}"#
        );
        assert!(result.is_err());
    }

    // A well-formed payload must parse cleanly with optional fields absent.
    #[test]
    fn create_schema_accepts_valid_payload() {
        let json = r#"{"name":"Miami","slug":"miami","country_id":"00000000-0000-0000-0000-000000000001"}"#;
        assert!(serde_json::from_str::<CreateDomainSchema>(json).is_ok());
    }
}
