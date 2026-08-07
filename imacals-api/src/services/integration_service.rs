use sqlx::PgPool;
use uuid::Uuid;

use crate::models::attribute::CreateAttributeSchema;
use crate::models::integration::{CreateIntegrationSchema, Integration, IntegrationCategory};
use crate::repositories::attribute_repository::AttributeRepository;
use crate::repositories::integration_repository::IntegrationRepository;
use crate::utilities::encryption;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::integration_type_defs;

pub struct IntegrationService;

impl IntegrationService {
    // Creates an integration and all of its typed attributes in a single DB transaction.
    // Validates that required fields for the type are present before opening the transaction.
    pub async fn create(
        pool: &PgPool,
        created_by: &Uuid,
        organization_id: &Uuid,
        schema: &CreateIntegrationSchema,
        app_secret: &str,
    ) -> Result<Integration, ErrorBag> {
        // Validate required fields for typed integrations.
        let required = integration_type_defs::fields_for_type(&schema.integration_type);
        for req in required.iter().filter(|f| f.is_required) {
            let provided = schema
                .attributes
                .iter()
                .flatten()
                .any(|a| a.name == req.name && a.value.as_deref().map_or(false, |v| !v.is_empty()));
            if !provided {
                return Err(ErrorBag::Validation {
                    field: req.name.into(),
                    message: format!("{} is required", req.label),
                });
            }
        }

        // Category always comes from the type — a client can't file a row under the wrong family.
        let category = schema.integration_type.category();

        // An explicit is_enabled wins; otherwise the row goes live only when its family has no
        // provider yet. That keeps the DB's one-live-per-category rule from rejecting the second
        // provider someone (or the seed) adds.
        let is_enabled = match schema.is_enabled {
            Some(explicit) => explicit,
            None if category == IntegrationCategory::Other => true,
            None => !IntegrationRepository::category_has_enabled(pool, category)
                .await
                .map_err(|e| {
                    ErrorBag::InternalServerError(format!(
                        "IntegrationService::create category probe: {e:?}"
                    ))
                })?,
        };

        let mut tx = pool.begin().await.map_err(|e| {
            ErrorBag::InternalServerError(format!("IntegrationService::create tx begin: {e:?}"))
        })?;

        let integration = IntegrationRepository::create(
            &mut *tx,
            created_by,
            organization_id,
            schema,
            category,
            is_enabled,
        )
        .await
        .map_err(|e| {
            ErrorBag::InternalServerError(format!("IntegrationService::create insert failed: {e:?}"))
        })?;

        for attr in schema.attributes.iter().flatten() {
            let mut value = attr.value.clone();
            if attr.is_encrypted == Some(true) {
                if let Some(ref plaintext) = value {
                    value = Some(
                        encryption::encrypt(plaintext, app_secret)
                            .map_err(|e| ErrorBag::InternalServerError(format!("encryption: {e}")))?,
                    );
                }
            }
            let attr_schema = CreateAttributeSchema {
                attributeable_type: "integrations".into(),
                attributeable_id: integration.id,
                name: attr.name.clone(),
                value,
                attribute_type: attr.attribute_type.clone(),
                is_encrypted: attr.is_encrypted,
            };
            AttributeRepository::create(&mut *tx, created_by, &attr_schema)
                .await
                .map_err(|e| {
                    ErrorBag::InternalServerError(format!(
                        "IntegrationService::create attr insert: {e:?}"
                    ))
                })?;
        }

        tx.commit().await.map_err(|e| {
            ErrorBag::InternalServerError(format!("IntegrationService::create commit: {e:?}"))
        })?;

        Ok(integration)
    }

    // Switches which provider of a family is live. Disabling siblings and enabling the target
    // happen in one transaction, so no window exists where two providers look live.
    pub async fn set_enabled(
        pool: &PgPool,
        id: &Uuid,
        enabled: bool,
    ) -> Result<Integration, ErrorBag> {
        let existing = IntegrationRepository::find_by_id(pool, id).await.map_err(|e| match e {
            sqlx::Error::RowNotFound => ErrorBag::NotFound("Integration".into()),
            other => ErrorBag::InternalServerError(format!(
                "IntegrationService::set_enabled lookup: {other:?}"
            )),
        })?;

        let mut tx = pool.begin().await.map_err(|e| {
            ErrorBag::InternalServerError(format!("IntegrationService::set_enabled tx begin: {e:?}"))
        })?;

        // Custom rows are exempt from the one-live rule, so leave their siblings alone.
        if enabled && existing.integration_category != IntegrationCategory::Other {
            IntegrationRepository::disable_siblings(&mut *tx, id).await.map_err(|e| {
                ErrorBag::InternalServerError(format!(
                    "IntegrationService::set_enabled disable siblings: {e:?}"
                ))
            })?;
        }

        IntegrationRepository::set_enabled(&mut *tx, id, enabled).await.map_err(|e| {
            ErrorBag::InternalServerError(format!("IntegrationService::set_enabled update: {e:?}"))
        })?;

        tx.commit().await.map_err(|e| {
            ErrorBag::InternalServerError(format!("IntegrationService::set_enabled commit: {e:?}"))
        })?;

        IntegrationRepository::find_by_id(pool, id).await.map_err(|e| {
            ErrorBag::InternalServerError(format!("IntegrationService::set_enabled reload: {e:?}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::integration::{InlineAttributeSchema, IntegrationType};
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

    fn smtp_schema(domain_id: Uuid, name: &str, slug: &str) -> CreateIntegrationSchema {
        CreateIntegrationSchema {
            organization_id: None,
            domain_id,
            name: name.into(),
            slug: slug.into(),
            integration_type: IntegrationType::Smtp,
            is_enabled: None,
            attributes: Some(vec![
                inline("SMTP_HOST", "imacals-mail", "text", false),
                inline("SMTP_PORT", "1025", "text", false),
                inline("SMTP_FROM_EMAIL", "no-reply@imacals.local", "text", false),
                inline("SMTP_PASSWORD", "s3cret", "password", true),
            ]),
        }
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_derives_category_from_type(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "svc-cat@test.com").await;
        let created = IntegrationService::create(
            &pool,
            &user_id,
            &org_id,
            &smtp_schema(domain_id, "Relay", "svc-relay"),
            "test-secret",
        )
        .await
        .unwrap();
        assert_eq!(created.integration_category, IntegrationCategory::Email);
    }

    // The first provider of a family goes live so a fresh install can send; later ones don't,
    // which is also what keeps the unique index from rejecting them.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn first_provider_is_live_and_the_second_is_not(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "svc-first@test.com").await;

        let first = IntegrationService::create(
            &pool,
            &user_id,
            &org_id,
            &smtp_schema(domain_id, "First", "svc-first-relay"),
            "test-secret",
        )
        .await
        .unwrap();
        assert!(first.is_enabled);

        let second = IntegrationService::create(
            &pool,
            &user_id,
            &org_id,
            &smtp_schema(domain_id, "Second", "svc-second-relay"),
            "test-secret",
        )
        .await
        .unwrap();
        assert!(!second.is_enabled, "the second Email provider must not auto-enable");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn missing_required_field_is_a_validation_error(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "svc-missing@test.com").await;
        let schema = CreateIntegrationSchema {
            organization_id: None,
            domain_id,
            name: "Incomplete".into(),
            slug: "svc-incomplete".into(),
            integration_type: IntegrationType::Mailgun,
            is_enabled: None,
            // MAILGUN_DOMAIN and MAILGUN_FROM_EMAIL are missing.
            attributes: Some(vec![inline("MAILGUN_API_KEY", "key-123", "password", true)]),
        };
        let result =
            IntegrationService::create(&pool, &user_id, &org_id, &schema, "test-secret").await;
        assert!(matches!(result, Err(ErrorBag::Validation { .. })));
    }

    // A rejected create must leave nothing behind — row and attributes share one transaction.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn failed_create_persists_nothing(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "svc-atomic@test.com").await;
        let schema = CreateIntegrationSchema {
            organization_id: None,
            domain_id,
            name: "Incomplete".into(),
            slug: "svc-atomic-slug".into(),
            integration_type: IntegrationType::Mailgun,
            is_enabled: None,
            attributes: Some(vec![inline("MAILGUN_API_KEY", "key-123", "password", true)]),
        };
        let _ = IntegrationService::create(&pool, &user_id, &org_id, &schema, "test-secret").await;
        assert!(matches!(
            IntegrationRepository::find_by_slug(&pool, "svc-atomic-slug").await,
            Err(sqlx::Error::RowNotFound)
        ));
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn password_attributes_are_encrypted_at_rest(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "svc-enc@test.com").await;
        let created = IntegrationService::create(
            &pool,
            &user_id,
            &org_id,
            &smtp_schema(domain_id, "Enc", "svc-enc-relay"),
            "test-secret",
        )
        .await
        .unwrap();

        let attrs = AttributeRepository::find_for_owner(&pool, "integrations", &created.id)
            .await
            .unwrap();
        let password = attrs.iter().find(|a| a.name == "SMTP_PASSWORD").unwrap();
        assert_ne!(
            password.value.as_deref(),
            Some("s3cret"),
            "the plaintext password must never reach the DB"
        );
        assert_eq!(
            encryption::decrypt(password.value.as_deref().unwrap(), "test-secret").unwrap(),
            "s3cret"
        );
    }

    // Switching providers is the whole point of storing credentials in the DB.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn set_enabled_switches_the_live_provider(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "svc-switch@test.com").await;

        let first = IntegrationService::create(
            &pool,
            &user_id,
            &org_id,
            &smtp_schema(domain_id, "Old", "svc-switch-old"),
            "test-secret",
        )
        .await
        .unwrap();
        let second = IntegrationService::create(
            &pool,
            &user_id,
            &org_id,
            &smtp_schema(domain_id, "New", "svc-switch-new"),
            "test-secret",
        )
        .await
        .unwrap();

        let switched = IntegrationService::set_enabled(&pool, &second.id, true).await.unwrap();
        assert!(switched.is_enabled);
        assert!(!IntegrationRepository::find_by_id(&pool, &first.id).await.unwrap().is_enabled);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn set_enabled_on_unknown_id_is_not_found(pool: PgPool) {
        let result = IntegrationService::set_enabled(&pool, &Uuid::new_v4(), true).await;
        assert!(matches!(result, Err(ErrorBag::NotFound(_))));
    }
}
