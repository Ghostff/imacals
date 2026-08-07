// Resolves live provider credentials at the moment of use.
//
// This is the seam that makes environment variables seed-only. `integration_seed` copies MAIL_* /
// MAILGUN_* / … out of the environment ONCE, into `integrations` + `attributes` rows. Everything
// afterwards — every send, every verification — comes through here, which re-reads the database on
// every call. So an admin editing credentials or switching providers in the dashboard changes
// behaviour immediately: no redeploy, no restart, no env edit.
//
// Nothing here is cached, deliberately. A process-lifetime cache would reintroduce exactly the
// restart-to-apply problem this design exists to remove; a send is already network-bound, so one
// indexed query per send is not the cost worth optimising.

use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::config::ENV;
use crate::models::integration::{Integration, IntegrationCategory};
use crate::repositories::attribute_repository::AttributeRepository;
use crate::repositories::integration_repository::IntegrationRepository;
use crate::utilities::encryption;
use crate::utilities::error_bag::ErrorBag;

// A live provider plus its decrypted credentials, ready to hand to a sender.
pub struct ResolvedIntegration {
    pub integration: Integration,
    pub values: HashMap<String, String>,
}

impl ResolvedIntegration {
    // For credentials the provider cannot work without.
    pub fn required(&self, name: &str) -> Result<String, ErrorBag> {
        match self.values.get(name) {
            Some(value) if !value.is_empty() => Ok(value.clone()),
            _ => Err(ErrorBag::Validation {
                field: name.to_string(),
                message: format!(
                    "{} is not configured on integration '{}'",
                    name, self.integration.slug
                ),
            }),
        }
    }

    // For credentials with a sensible default (region, TLS flag, display name).
    pub fn optional(&self, name: &str) -> Option<String> {
        self.values.get(name).filter(|v| !v.is_empty()).cloned()
    }
}

pub struct IntegrationResolverService;

impl IntegrationResolverService {
    // The provider a category is currently sending through, with credentials attached.
    // NotFound means nothing is live for that family — callers should surface "no provider
    // configured" rather than silently doing nothing.
    pub async fn resolve(
        pool: &PgPool,
        category: IntegrationCategory,
    ) -> Result<ResolvedIntegration, ErrorBag> {
        let integration = IntegrationRepository::find_enabled_by_category(pool, category)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => ErrorBag::NotFound(format!("{category:?} provider")),
                other => ErrorBag::InternalServerError(format!(
                    "IntegrationResolverService::resolve lookup: {other:?}"
                )),
            })?;

        let values = Self::values_for(pool, &integration.id).await?;
        Ok(ResolvedIntegration { integration, values })
    }

    // Same as `resolve`, but for a specific row — used when a campaign pins a provider instead of
    // following whichever one is live.
    pub async fn resolve_by_id(pool: &PgPool, id: &Uuid) -> Result<ResolvedIntegration, ErrorBag> {
        let integration = IntegrationRepository::find_by_id(pool, id).await.map_err(|e| match e {
            sqlx::Error::RowNotFound => ErrorBag::NotFound("Integration".into()),
            other => ErrorBag::InternalServerError(format!(
                "IntegrationResolverService::resolve_by_id lookup: {other:?}"
            )),
        })?;

        if !integration.is_enabled {
            return Err(ErrorBag::Validation {
                field: "integration_id".into(),
                message: format!("Integration '{}' is disabled", integration.slug),
            });
        }

        let values = Self::values_for(pool, &integration.id).await?;
        Ok(ResolvedIntegration { integration, values })
    }

    // Attribute name -> plaintext value. Encrypted attributes are decrypted here so no caller
    // needs to know which fields were stored as ciphertext.
    pub async fn values_for(
        pool: &PgPool,
        integration_id: &Uuid,
    ) -> Result<HashMap<String, String>, ErrorBag> {
        let attributes = AttributeRepository::find_for_owner(pool, "integrations", integration_id)
            .await
            .map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "IntegrationResolverService::values_for load: {e:?}"
                ))
            })?;

        let mut values = HashMap::with_capacity(attributes.len());
        for attribute in attributes {
            let Some(raw) = attribute.value else { continue };
            let plaintext = if attribute.is_encrypted {
                encryption::decrypt(&raw, &ENV.app_secret).map_err(|e| {
                    // Names the attribute but never the value — this error reaches the logs.
                    ErrorBag::InternalServerError(format!(
                        "IntegrationResolverService could not decrypt '{}': {e}",
                        attribute.name
                    ))
                })?
            } else {
                raw
            };
            values.insert(attribute.name, plaintext);
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::integration::{
        CreateIntegrationSchema, InlineAttributeSchema, IntegrationType,
    };
    use crate::services::integration_service::IntegrationService;
    use sqlx::PgPool;

    async fn default_org_id(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!("SELECT id FROM organizations WHERE slug = 'imacals' LIMIT 1")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn default_domain_id(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!("SELECT id FROM domains WHERE slug = 'default-us' LIMIT 1")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn make_user(pool: &PgPool, email: &str) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T',$1,'x',NOW()) RETURNING id",
            email
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn inline(name: &str, value: &str, attr_type: &str, encrypted: bool) -> InlineAttributeSchema {
        InlineAttributeSchema {
            name: name.into(),
            value: Some(value.into()),
            attribute_type: attr_type.into(),
            is_encrypted: Some(encrypted),
        }
    }

    // Builds an SMTP provider whose password is encrypted with the running APP_SECRET, so the
    // resolver's decrypt path is exercised for real.
    async fn seed_smtp(
        pool: &PgPool,
        slug: &str,
        host: &str,
        password: &str,
        enabled: Option<bool>,
    ) -> Integration {
        let org_id = default_org_id(pool).await;
        let domain_id = default_domain_id(pool).await;
        let user_id = make_user(pool, &format!("{slug}@test.com")).await;
        let schema = CreateIntegrationSchema {
            organization_id: None,
            domain_id,
            name: slug.into(),
            slug: slug.into(),
            integration_type: IntegrationType::Smtp,
            is_enabled: enabled,
            attributes: Some(vec![
                inline("SMTP_HOST", host, "text", false),
                inline("SMTP_PORT", "1025", "text", false),
                inline("SMTP_FROM_EMAIL", "no-reply@imacals.local", "text", false),
                inline("SMTP_PASSWORD", password, "password", true),
            ]),
        };
        IntegrationService::create(pool, &user_id, &org_id, &schema, &ENV.app_secret)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn resolve_returns_the_live_provider_with_decrypted_values(pool: PgPool) {
        seed_smtp(&pool, "resolver-live", "imacals-mail", "s3cret", Some(true)).await;

        let resolved = IntegrationResolverService::resolve(&pool, IntegrationCategory::Email)
            .await
            .unwrap();
        assert_eq!(resolved.integration.slug, "resolver-live");
        assert_eq!(resolved.required("SMTP_HOST").unwrap(), "imacals-mail");
        // The password was stored as ciphertext; the resolver hands back plaintext.
        assert_eq!(resolved.required("SMTP_PASSWORD").unwrap(), "s3cret");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn resolve_without_a_live_provider_is_not_found(pool: PgPool) {
        seed_smtp(&pool, "resolver-off", "imacals-mail", "s3cret", Some(false)).await;
        assert!(matches!(
            IntegrationResolverService::resolve(&pool, IntegrationCategory::Email).await,
            Err(ErrorBag::NotFound(_))
        ));
    }

    // The whole point: an edit through the API changes what the next send uses, with no restart.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn an_attribute_edit_is_visible_to_the_next_resolve(pool: PgPool) {
        let integration = seed_smtp(&pool, "resolver-edit", "old-host", "s3cret", Some(true)).await;

        let before = IntegrationResolverService::resolve(&pool, IntegrationCategory::Email)
            .await
            .unwrap();
        assert_eq!(before.required("SMTP_HOST").unwrap(), "old-host");

        let attrs = AttributeRepository::find_for_owner(&pool, "integrations", &integration.id)
            .await
            .unwrap();
        let host = attrs.iter().find(|a| a.name == "SMTP_HOST").unwrap();
        sqlx::query!(
            "UPDATE attributes SET value = 'new-host', updated_at = NOW() WHERE id = $1",
            host.id
        )
        .execute(&pool)
        .await
        .unwrap();

        let after = IntegrationResolverService::resolve(&pool, IntegrationCategory::Email)
            .await
            .unwrap();
        assert_eq!(
            after.required("SMTP_HOST").unwrap(),
            "new-host",
            "the resolver must re-read the DB, not cache the boot-time value"
        );
    }

    // Switching the live provider must change what resolve() returns on the very next call.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn switching_providers_changes_what_resolve_returns(pool: PgPool) {
        seed_smtp(&pool, "resolver-first", "host-one", "s3cret", Some(true)).await;
        let second = seed_smtp(&pool, "resolver-second", "host-two", "s3cret", Some(false)).await;

        IntegrationService::set_enabled(&pool, &second.id, true).await.unwrap();

        let resolved = IntegrationResolverService::resolve(&pool, IntegrationCategory::Email)
            .await
            .unwrap();
        assert_eq!(resolved.integration.slug, "resolver-second");
        assert_eq!(resolved.required("SMTP_HOST").unwrap(), "host-two");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn required_reports_a_missing_credential_as_validation(pool: PgPool) {
        seed_smtp(&pool, "resolver-missing", "imacals-mail", "s3cret", Some(true)).await;
        let resolved = IntegrationResolverService::resolve(&pool, IntegrationCategory::Email)
            .await
            .unwrap();
        assert!(matches!(
            resolved.required("SMTP_NOT_SET"),
            Err(ErrorBag::Validation { .. })
        ));
        assert!(resolved.optional("SMTP_NOT_SET").is_none());
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn resolve_by_id_rejects_a_disabled_provider(pool: PgPool) {
        let disabled = seed_smtp(&pool, "resolver-pinned", "imacals-mail", "s3cret", Some(false)).await;
        assert!(matches!(
            IntegrationResolverService::resolve_by_id(&pool, &disabled.id).await,
            Err(ErrorBag::Validation { .. })
        ));
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn resolve_by_id_on_unknown_id_is_not_found(pool: PgPool) {
        assert!(matches!(
            IntegrationResolverService::resolve_by_id(&pool, &Uuid::new_v4()).await,
            Err(ErrorBag::NotFound(_))
        ));
    }

    // Categories must not bleed: a live verifier is not a sender.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn a_verifier_is_never_resolved_as_a_sender(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "resolver-verifier@test.com").await;
        let schema = CreateIntegrationSchema {
            organization_id: None,
            domain_id,
            name: "ZeroBounce".into(),
            slug: "resolver-zerobounce".into(),
            integration_type: IntegrationType::ZeroBounce,
            is_enabled: Some(true),
            attributes: Some(vec![inline("ZEROBOUNCE_API_KEY", "zb-key", "password", true)]),
        };
        IntegrationService::create(&pool, &user_id, &org_id, &schema, &ENV.app_secret)
            .await
            .unwrap();

        assert!(matches!(
            IntegrationResolverService::resolve(&pool, IntegrationCategory::Email).await,
            Err(ErrorBag::NotFound(_))
        ));
        let verifier =
            IntegrationResolverService::resolve(&pool, IntegrationCategory::EmailValidation)
                .await
                .unwrap();
        assert_eq!(verifier.required("ZEROBOUNCE_API_KEY").unwrap(), "zb-key");
    }
}
