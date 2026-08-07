use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::models::integration::{
    CreateIntegrationSchema, Integration, IntegrationCategory, IntegrationType,
    UpdateIntegrationSchema,
};

// Every SELECT repeats the same column list: the `AS "col: Type"` casts are what let sqlx map the
// text columns onto the enums, and query_as! needs them written literally (a macro-expanded string
// isn't visible to it at compile time).
// IntegrationRepository is the only place that talks to the integrations table.
pub struct IntegrationRepository;

impl IntegrationRepository {
    pub async fn index(pool: &PgPool) -> Result<Vec<Integration>, Error> {
        Ok(sqlx::query_as!(
            Integration,
            r#"SELECT id, organization_id, domain_id, created_by, name, slug,
                      integration_type     AS "integration_type: IntegrationType",
                      integration_category AS "integration_category: IntegrationCategory",
                      is_enabled, created_at, updated_at, deleted_at
               FROM integrations
               WHERE deleted_at IS NULL
               ORDER BY integration_category ASC, name ASC"#
        )
        .fetch_all(pool)
        .await?)
    }

    // Powers the dashboard's per-family listing (all Email providers, all verifiers, …).
    pub async fn index_for_category(
        pool: &PgPool,
        category: IntegrationCategory,
    ) -> Result<Vec<Integration>, Error> {
        Ok(sqlx::query_as!(
            Integration,
            r#"SELECT id, organization_id, domain_id, created_by, name, slug,
                      integration_type     AS "integration_type: IntegrationType",
                      integration_category AS "integration_category: IntegrationCategory",
                      is_enabled, created_at, updated_at, deleted_at
               FROM integrations
               WHERE integration_category = $1 AND deleted_at IS NULL
               ORDER BY is_enabled DESC, name ASC"#,
            category as IntegrationCategory,
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Integration, Error> {
        Ok(sqlx::query_as!(
            Integration,
            r#"SELECT id, organization_id, domain_id, created_by, name, slug,
                      integration_type     AS "integration_type: IntegrationType",
                      integration_category AS "integration_category: IntegrationCategory",
                      is_enabled, created_at, updated_at, deleted_at
               FROM integrations
               WHERE id = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    // The resolver's lookup: who is live for this family right now. Read per use — never cached —
    // so an admin flipping providers in the dashboard takes effect on the next call.
    pub async fn find_enabled_by_category(
        pool: &PgPool,
        category: IntegrationCategory,
    ) -> Result<Integration, Error> {
        Ok(sqlx::query_as!(
            Integration,
            r#"SELECT id, organization_id, domain_id, created_by, name, slug,
                      integration_type     AS "integration_type: IntegrationType",
                      integration_category AS "integration_category: IntegrationCategory",
                      is_enabled, created_at, updated_at, deleted_at
               FROM integrations
               WHERE integration_category = $1
                 AND is_enabled = TRUE
                 AND deleted_at IS NULL
               ORDER BY updated_at DESC
               LIMIT 1"#,
            category as IntegrationCategory,
        )
        .fetch_one(pool)
        .await?)
    }

    // True when a family already has a live provider. The seed uses this to insert additional
    // providers disabled instead of tripping the one-enabled-per-category unique index.
    pub async fn category_has_enabled(
        pool: &PgPool,
        category: IntegrationCategory,
    ) -> Result<bool, Error> {
        Ok(sqlx::query_scalar!(
            r#"SELECT EXISTS(
                   SELECT 1 FROM integrations
                   WHERE integration_category = $1
                     AND is_enabled = TRUE
                     AND deleted_at IS NULL
               )"#,
            category as IntegrationCategory,
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(false))
    }

    pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Integration, Error> {
        Ok(sqlx::query_as!(
            Integration,
            r#"SELECT id, organization_id, domain_id, created_by, name, slug,
                      integration_type     AS "integration_type: IntegrationType",
                      integration_category AS "integration_category: IntegrationCategory",
                      is_enabled, created_at, updated_at, deleted_at
               FROM integrations
               WHERE slug = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            slug
        )
        .fetch_one(pool)
        .await?)
    }

    // Accepts any PgExecutor so it can be called within a transaction from the service layer.
    // `category` and `is_enabled` come from the service, never the client: the category is derived
    // from the type, and the enabled decision depends on whether the family already has a winner.
    pub async fn create<'e>(
        executor: impl sqlx::PgExecutor<'e>,
        created_by: &Uuid,
        organization_id: &Uuid,
        schema: &CreateIntegrationSchema,
        category: IntegrationCategory,
        is_enabled: bool,
    ) -> Result<Integration, Error> {
        Ok(sqlx::query_as!(
            Integration,
            r#"INSERT INTO integrations
                   (organization_id, domain_id, created_by, name, slug,
                    integration_type, integration_category, is_enabled)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING id, organization_id, domain_id, created_by, name, slug,
                         integration_type     AS "integration_type: IntegrationType",
                         integration_category AS "integration_category: IntegrationCategory",
                         is_enabled, created_at, updated_at, deleted_at"#,
            organization_id,
            schema.domain_id,
            created_by,
            schema.name,
            schema.slug,
            schema.integration_type as IntegrationType,
            category as IntegrationCategory,
            is_enabled,
        )
        .fetch_one(executor)
        .await?)
    }

    // Changing the type re-derives the category in the same statement, so the two columns can
    // never drift apart.
    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        schema: &UpdateIntegrationSchema,
    ) -> Result<Integration, Error> {
        let category = schema.integration_type.map(|t| t.category());

        Ok(sqlx::query_as!(
            Integration,
            r#"UPDATE integrations
               SET organization_id      = COALESCE($2, organization_id),
                   domain_id            = COALESCE($3, domain_id),
                   name                 = COALESCE($4, name),
                   slug                 = COALESCE($5, slug),
                   integration_type     = COALESCE($6, integration_type),
                   integration_category = COALESCE($7, integration_category),
                   updated_at           = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, organization_id, domain_id, created_by, name, slug,
                         integration_type     AS "integration_type: IntegrationType",
                         integration_category AS "integration_category: IntegrationCategory",
                         is_enabled, created_at, updated_at, deleted_at"#,
            id,
            schema.organization_id,
            schema.domain_id,
            schema.name,
            schema.slug,
            schema.integration_type as Option<IntegrationType>,
            category as Option<IntegrationCategory>,
        )
        .fetch_one(pool)
        .await?)
    }

    // Disables every other provider in the same family. Runs inside the switch transaction so no
    // window exists where two providers look live.
    pub async fn disable_siblings<'e>(
        executor: impl sqlx::PgExecutor<'e>,
        id: &Uuid,
    ) -> Result<u64, Error> {
        Ok(sqlx::query!(
            r#"UPDATE integrations AS siblings
               SET is_enabled = FALSE, updated_at = NOW()
               FROM integrations AS target
               WHERE target.id = $1
                 AND siblings.id <> target.id
                 AND siblings.organization_id      = target.organization_id
                 AND siblings.domain_id            = target.domain_id
                 AND siblings.integration_category = target.integration_category
                 AND siblings.is_enabled = TRUE
                 AND siblings.deleted_at IS NULL"#,
            id
        )
        .execute(executor)
        .await?
        .rows_affected())
    }

    pub async fn set_enabled<'e>(
        executor: impl sqlx::PgExecutor<'e>,
        id: &Uuid,
        enabled: bool,
    ) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE integrations SET is_enabled = $2, updated_at = NOW()
             WHERE id = $1 AND deleted_at IS NULL",
            id,
            enabled
        )
        .execute(executor)
        .await?
        .rows_affected())
    }

    // Soft-delete: attributes are cascaded by the DB trigger.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE integrations SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn make_schema(
        domain_id: Uuid,
        name: &str,
        slug: &str,
        integration_type: IntegrationType,
    ) -> CreateIntegrationSchema {
        CreateIntegrationSchema {
            organization_id: None,
            domain_id,
            name: name.into(),
            slug: slug.into(),
            integration_type,
            is_enabled: None,
            attributes: None,
        }
    }

    // Inserts a row the way the service would: category derived from the type.
    async fn insert(
        pool: &PgPool,
        user_id: &Uuid,
        org_id: &Uuid,
        schema: &CreateIntegrationSchema,
        enabled: bool,
    ) -> Integration {
        let category = schema.integration_type.category();
        IntegrationRepository::create(pool, user_id, org_id, schema, category, enabled)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn created_integration_appears_in_index(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-index@test.com").await;
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Campaign Relay", "campaign-relay", IntegrationType::Smtp),
            true,
        )
        .await;
        let rows = IntegrationRepository::index(&pool).await.unwrap();
        assert!(rows.iter().any(|r| r.slug == "campaign-relay"));
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleted_integration_is_hidden(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-del@test.com").await;
        let integration = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Gone", "gone-int", IntegrationType::Custom),
            true,
        )
        .await;
        IntegrationRepository::delete(&pool, &integration.id).await.unwrap();
        let rows = IntegrationRepository::index(&pool).await.unwrap();
        assert!(
            !rows.iter().any(|r| r.id == integration.id),
            "soft-deleted integration must not appear"
        );
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn double_delete_returns_zero(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-dup@test.com").await;
        let integration = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Dup", "dup-int", IntegrationType::Custom),
            true,
        )
        .await;
        IntegrationRepository::delete(&pool, &integration.id).await.unwrap();
        let affected = IntegrationRepository::delete(&pool, &integration.id).await.unwrap();
        assert_eq!(affected, 0);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn find_by_id_returns_correct_integration(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-find@test.com").await;
        let created = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Find Me", "find-me-int", IntegrationType::Custom),
            true,
        )
        .await;
        let found = IntegrationRepository::find_by_id(&pool, &created.id).await.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.name, "Find Me");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn find_by_id_after_delete_is_not_found(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-gone@test.com").await;
        let integration = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Gone2", "gone2-int", IntegrationType::Custom),
            true,
        )
        .await;
        IntegrationRepository::delete(&pool, &integration.id).await.unwrap();
        assert!(matches!(
            IntegrationRepository::find_by_id(&pool, &integration.id).await,
            Err(sqlx::Error::RowNotFound)
        ));
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn update_changes_name(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-upd@test.com").await;
        let integration = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Old", "old-int", IntegrationType::Custom),
            true,
        )
        .await;
        let schema = UpdateIntegrationSchema {
            organization_id: None,
            domain_id: None,
            name: Some("New Name".into()),
            slug: None,
            integration_type: None,
        };
        let updated = IntegrationRepository::update(&pool, &integration.id, &schema)
            .await
            .unwrap();
        assert_eq!(updated.name, "New Name");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn created_integration_has_correct_type(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-type@test.com").await;
        let created = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Mailgun", "mailgun-type-test", IntegrationType::Mailgun),
            true,
        )
        .await;
        assert_eq!(created.integration_type, IntegrationType::Mailgun);
        assert_eq!(created.integration_category, IntegrationCategory::Email);
    }

    // Changing the type must drag the category with it, or the row becomes invisible to the
    // resolver for its new family.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn update_type_rederives_category(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-recat@test.com").await;
        let integration = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Was Custom", "was-custom", IntegrationType::Custom),
            false,
        )
        .await;
        assert_eq!(integration.integration_category, IntegrationCategory::Other);

        let schema = UpdateIntegrationSchema {
            organization_id: None,
            domain_id: None,
            name: None,
            slug: None,
            integration_type: Some(IntegrationType::ZeroBounce),
        };
        let updated = IntegrationRepository::update(&pool, &integration.id, &schema)
            .await
            .unwrap();
        assert_eq!(updated.integration_category, IntegrationCategory::EmailValidation);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn find_enabled_by_category_returns_the_live_provider(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-live@test.com").await;
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Disabled Relay", "disabled-relay", IntegrationType::Smtp),
            false,
        )
        .await;
        let live = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Live Relay", "live-relay", IntegrationType::Mailgun),
            true,
        )
        .await;

        let found = IntegrationRepository::find_enabled_by_category(&pool, IntegrationCategory::Email)
            .await
            .unwrap();
        assert_eq!(found.id, live.id);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn find_enabled_by_category_is_not_found_when_nothing_is_live(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-nolive@test.com").await;
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Off", "off-relay", IntegrationType::Smtp),
            false,
        )
        .await;
        assert!(matches!(
            IntegrationRepository::find_enabled_by_category(&pool, IntegrationCategory::Email).await,
            Err(sqlx::Error::RowNotFound)
        ));
    }

    // The DB, not the application, is what guarantees a single live sender.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn two_enabled_providers_in_one_category_are_rejected(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-conflict@test.com").await;
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "First", "first-relay", IntegrationType::Smtp),
            true,
        )
        .await;
        let second = IntegrationRepository::create(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Second", "second-relay", IntegrationType::Mailgun),
            IntegrationCategory::Email,
            true,
        )
        .await;
        assert!(second.is_err(), "a second live Email provider must violate the unique index");
    }

    // Two Custom rows can both be enabled — 'other' is exempt from the one-live rule.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn custom_rows_are_exempt_from_the_one_enabled_rule(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-custom-two@test.com").await;
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Custom A", "custom-a", IntegrationType::Custom),
            true,
        )
        .await;
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Custom B", "custom-b", IntegrationType::Custom),
            true,
        )
        .await;
        let rows = IntegrationRepository::index_for_category(&pool, IntegrationCategory::Other)
            .await
            .unwrap();
        assert_eq!(rows.iter().filter(|r| r.is_enabled).count(), 2);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn disable_siblings_leaves_only_the_target_live(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-switch@test.com").await;
        let first = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Old Sender", "old-sender", IntegrationType::Smtp),
            true,
        )
        .await;
        let second = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "New Sender", "new-sender", IntegrationType::Mailgun),
            false,
        )
        .await;

        IntegrationRepository::disable_siblings(&pool, &second.id).await.unwrap();
        IntegrationRepository::set_enabled(&pool, &second.id, true).await.unwrap();

        assert!(!IntegrationRepository::find_by_id(&pool, &first.id).await.unwrap().is_enabled);
        let live = IntegrationRepository::find_enabled_by_category(&pool, IntegrationCategory::Email)
            .await
            .unwrap();
        assert_eq!(live.id, second.id);
    }

    // Categories are independent: a verifier being live says nothing about the sender family.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn disable_siblings_does_not_cross_categories(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-crosscat@test.com").await;
        let sender = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Sender", "cat-sender", IntegrationType::Smtp),
            true,
        )
        .await;
        let verifier = insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Verifier", "cat-verifier", IntegrationType::ZeroBounce),
            true,
        )
        .await;

        IntegrationRepository::disable_siblings(&pool, &verifier.id).await.unwrap();
        assert!(
            IntegrationRepository::find_by_id(&pool, &sender.id).await.unwrap().is_enabled,
            "enabling a verifier must not disable the sender"
        );
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn category_has_enabled_reflects_the_live_row(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-hasenabled@test.com").await;
        assert!(!IntegrationRepository::category_has_enabled(&pool, IntegrationCategory::Email)
            .await
            .unwrap());
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Relay", "has-enabled-relay", IntegrationType::Smtp),
            true,
        )
        .await;
        assert!(IntegrationRepository::category_has_enabled(&pool, IntegrationCategory::Email)
            .await
            .unwrap());
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn index_for_category_only_returns_that_family(pool: PgPool) {
        let org_id = default_org_id(&pool).await;
        let domain_id = default_domain_id(&pool).await;
        let user_id = make_user(&pool, "int-cat-index@test.com").await;
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Relay", "cat-index-relay", IntegrationType::Smtp),
            true,
        )
        .await;
        insert(
            &pool,
            &user_id,
            &org_id,
            &make_schema(domain_id, "Verifier", "cat-index-verifier", IntegrationType::ZeroBounce),
            true,
        )
        .await;

        let email = IntegrationRepository::index_for_category(&pool, IntegrationCategory::Email)
            .await
            .unwrap();
        assert!(email
            .iter()
            .all(|r| r.integration_category == IntegrationCategory::Email));
        assert!(email.iter().any(|r| r.slug == "cat-index-relay"));
        assert!(!email.iter().any(|r| r.slug == "cat-index-verifier"));
    }
}
