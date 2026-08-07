use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Raw DB row — one active system-user assignment per (domain_id, user_role_id).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSystemUser {
    pub id:           Uuid,
    pub domain_id:    Uuid,
    pub user_id:      Uuid,
    pub user_role_id: Uuid,
    pub created_by:   Uuid,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at:   Option<DateTime<Utc>>,
}

// Enriched view returned by the API — joins user and role names for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSystemUserView {
    pub id:              Uuid,
    pub domain_id:       Uuid,
    pub domain_name:     String,
    pub user_id:         Uuid,
    pub user_first_name: String,
    pub user_last_name:  String,
    pub user_email:      String,
    pub user_role_id:    Uuid,
    pub role_name:       String,
    pub role_title:      String,
    pub created_by:      Uuid,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at:      Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDomainSystemUserSchema {
    pub domain_id:    Uuid,
    pub user_id:      Uuid,
    // Must reference a role where system_user_eligible = TRUE — validated in the controller.
    pub user_role_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_schema_requires_all_three_fields() {
        let missing_role: Result<CreateDomainSystemUserSchema, _> =
            serde_json::from_str(r#"{"domain_id":"00000000-0000-0000-0000-000000000001","user_id":"00000000-0000-0000-0000-000000000002"}"#);
        assert!(missing_role.is_err());

        let missing_user: Result<CreateDomainSystemUserSchema, _> =
            serde_json::from_str(r#"{"domain_id":"00000000-0000-0000-0000-000000000001","user_role_id":"00000000-0000-0000-0000-000000000003"}"#);
        assert!(missing_user.is_err());
    }

    #[test]
    fn create_schema_accepts_valid_payload() {
        let json = r#"{
            "domain_id":    "00000000-0000-0000-0000-000000000001",
            "user_id":      "00000000-0000-0000-0000-000000000002",
            "user_role_id": "00000000-0000-0000-0000-000000000003"
        }"#;
        assert!(serde_json::from_str::<CreateDomainSystemUserSchema>(json).is_ok());
    }

    #[test]
    fn view_deleted_at_skipped_when_none() {
        let v = DomainSystemUserView {
            id:              Uuid::nil(),
            domain_id:       Uuid::nil(),
            domain_name:     "Default US".into(),
            user_id:         Uuid::nil(),
            user_first_name: "Jane".into(),
            user_last_name:  "Doe".into(),
            user_email:      "jane@test.com".into(),
            user_role_id:    Uuid::nil(),
            role_name:       "broker".into(),
            role_title:      "Broker".into(),
            created_by:      Uuid::nil(),
            created_at:      DateTime::from_timestamp(0, 0).unwrap(),
            updated_at:      DateTime::from_timestamp(0, 0).unwrap(),
            deleted_at:      None,
        };
        let val = serde_json::to_value(&v).unwrap();
        assert!(val.get("deleted_at").is_none());
    }
}
