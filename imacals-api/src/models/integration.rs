use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Identifies which provider a connection speaks to.
// Custom means no predefined field template — the caller manages attributes freely.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationType {
    // Plain SMTP relay — also how dev talks to the Mailpit catcher (imacals-mail:1025).
    Smtp,
    // Writes the message to the API log instead of sending it. No credentials, no network:
    // the safe default so a fresh install can exercise a campaign without a provider.
    Log,
    Mailgun,
    Mailchimp,
    // Gmail via OAuth (installed-app refresh-token flow).
    Google,
    // Microsoft Graph client-credentials flow, sending as a specific mailbox.
    Outlook,
    // Address verification, run before a send so provably dead addresses are skipped —
    // bounces are what get a sending domain blocked.
    ZeroBounce,
    Custom,
}

// Groups providers that are interchangeable: any Email provider can send a campaign, so the
// resolver picks by category and doesn't care which vendor is behind it. Stored in
// integrations.integration_category and always derived from integration_type by the service —
// the two columns can never disagree.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationCategory {
    Email,
    EmailValidation,
    Other,
}

impl IntegrationType {
    pub fn category(&self) -> IntegrationCategory {
        match self {
            IntegrationType::Smtp
            | IntegrationType::Log
            | IntegrationType::Mailgun
            | IntegrationType::Mailchimp
            | IntegrationType::Google
            | IntegrationType::Outlook => IntegrationCategory::Email,
            IntegrationType::ZeroBounce => IntegrationCategory::EmailValidation,
            IntegrationType::Custom => IntegrationCategory::Other,
        }
    }
}

// A third-party provider connection scoped to an org and domain. Credentials live in the
// polymorphic `attributes` table, not on this row — see IntegrationResolverService for how they
// are read (and decrypted) at the moment of use rather than cached at boot.
// organization_id defaults to the "imacals" platform org when not specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub domain_id: Uuid,
    pub created_by: Uuid,
    pub name: String,
    pub slug: String,
    pub integration_type: IntegrationType,
    pub integration_category: IntegrationCategory,
    // Only one provider per category may be enabled at a time (DB-enforced) — this is the flag
    // an admin flips to switch senders.
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// An attribute sent inline with an integration create — no attributeable_type/id needed
// since they are always set to "integrations" and the new integration's id by the service.
#[derive(Debug, Clone, Deserialize)]
pub struct InlineAttributeSchema {
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub attribute_type: String,
    pub is_encrypted: Option<bool>,
}

// The form sent when creating an integration.
// organization_id is optional — the service resolves it to the "imacals" org when omitted.
// attributes are inserted atomically in the same transaction.
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateIntegrationSchema {
    pub organization_id: Option<Uuid>,
    pub domain_id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Name must be 1–255 characters"))]
    pub name: String,
    #[validate(length(min = 1, max = 255, message = "Slug must be 1–255 characters"))]
    pub slug: String,
    pub integration_type: IntegrationType,
    // Absent means "leave the switching decision to the admin": the service enables the row only
    // when its category has no live provider yet, so seeding several never trips the DB's
    // one-enabled-per-category rule.
    #[serde(default)]
    pub is_enabled: Option<bool>,
    pub attributes: Option<Vec<InlineAttributeSchema>>,
}

// PATCH semantics — every field optional.
// integration_category is deliberately absent: it is derived from integration_type, so a client
// can never post a row into the wrong family.
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateIntegrationSchema {
    pub organization_id: Option<Uuid>,
    pub domain_id: Option<Uuid>,
    #[validate(length(min = 1, max = 255, message = "Name must be 1–255 characters"))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255, message = "Slug must be 1–255 characters"))]
    pub slug: Option<String>,
    pub integration_type: Option<IntegrationType>,
}

// Body of the enable/disable toggle. Separate from UpdateIntegrationSchema because switching the
// live provider takes a different path: enabling one disables its siblings in the same category.
#[derive(Debug, Deserialize, Validate)]
pub struct SetEnabledSchema {
    pub is_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_schema_requires_domain_id() {
        let result: Result<CreateIntegrationSchema, _> =
            serde_json::from_str(r#"{"name":"SMTP","slug":"smtp","integration_type":"smtp"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn create_schema_requires_name() {
        let result: Result<CreateIntegrationSchema, _> = serde_json::from_str(
            r#"{"domain_id":"00000000-0000-0000-0000-000000000001","slug":"smtp","integration_type":"smtp"}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn create_schema_requires_slug() {
        let result: Result<CreateIntegrationSchema, _> = serde_json::from_str(
            r#"{"domain_id":"00000000-0000-0000-0000-000000000001","name":"SMTP","integration_type":"smtp"}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn create_schema_requires_integration_type() {
        let result: Result<CreateIntegrationSchema, _> = serde_json::from_str(
            r#"{"domain_id":"00000000-0000-0000-0000-000000000001","name":"SMTP","slug":"smtp"}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn create_schema_accepts_smtp_with_attributes() {
        let json = r#"{
            "domain_id":        "00000000-0000-0000-0000-000000000001",
            "name":             "Campaign Relay",
            "slug":             "campaign-relay",
            "integration_type": "smtp",
            "attributes": [
                { "name": "SMTP_HOST",     "value": "imacals-mail", "type": "text",     "is_encrypted": false },
                { "name": "SMTP_PASSWORD", "value": "secret",     "type": "password", "is_encrypted": true  }
            ]
        }"#;
        let parsed: CreateIntegrationSchema = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.integration_type, IntegrationType::Smtp);
        assert_eq!(parsed.attributes.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn create_schema_accepts_custom_without_attributes() {
        let json = r#"{
            "domain_id":        "00000000-0000-0000-0000-000000000001",
            "name":             "Custom Integration",
            "slug":             "custom-int",
            "integration_type": "custom"
        }"#;
        let parsed: CreateIntegrationSchema = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.integration_type, IntegrationType::Custom);
        assert!(parsed.attributes.is_none());
        assert!(parsed.is_enabled.is_none());
    }

    #[test]
    fn update_schema_all_fields_optional() {
        let parsed: UpdateIntegrationSchema = serde_json::from_str("{}").unwrap();
        assert!(parsed.name.is_none());
        assert!(parsed.slug.is_none());
        assert!(parsed.domain_id.is_none());
        assert!(parsed.integration_type.is_none());
    }

    // Every sending provider must land in Email so the resolver can treat them interchangeably.
    #[test]
    fn sending_providers_are_all_email_category() {
        for t in [
            IntegrationType::Smtp,
            IntegrationType::Log,
            IntegrationType::Mailgun,
            IntegrationType::Mailchimp,
            IntegrationType::Google,
            IntegrationType::Outlook,
        ] {
            assert_eq!(t.category(), IntegrationCategory::Email, "{t:?}");
        }
    }

    // Verification is deliberately its own category: a verifier selected as the sender would
    // silently break every campaign send.
    #[test]
    fn zero_bounce_is_not_a_sender() {
        assert_eq!(
            IntegrationType::ZeroBounce.category(),
            IntegrationCategory::EmailValidation
        );
    }

    #[test]
    fn type_serializes_kebab_case() {
        let json = serde_json::to_string(&IntegrationType::ZeroBounce).unwrap();
        assert_eq!(json, r#""zero-bounce""#);
    }

    #[test]
    fn set_enabled_schema_requires_the_flag() {
        assert!(serde_json::from_str::<SetEnabledSchema>("{}").is_err());
        assert!(serde_json::from_str::<SetEnabledSchema>(r#"{"is_enabled":false}"#).is_ok());
    }
}
