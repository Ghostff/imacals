// Seeds email providers from the environment — ONCE.
//
// The env vars below are a bootstrap convenience only: they exist so a fresh install comes up with
// a working sender instead of an empty integrations page. After the row exists, nothing reads the
// environment again — IntegrationResolverService reads the DB on every send, so credentials are
// changed in the dashboard, not in .env, and take effect without a restart. Editing an env var
// after the first boot therefore has NO effect; edit the integration instead.
//
// Idempotent: each provider is skipped when its slug already exists (including when an admin later
// renamed or disabled it). Providers whose env vars are absent are skipped entirely.

use std::env;

use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::config::ENV;
use crate::models::integration::{
    CreateIntegrationSchema, InlineAttributeSchema, IntegrationType,
};
use crate::services::integration_service::IntegrationService;

pub async fn run(pool: &PgPool) {
    let Some(domain_id) = resolve_domain(pool).await else {
        warn!("integration_seed: default-us domain not found — skipping");
        return;
    };

    let Some(org_id) = resolve_org(pool).await else {
        warn!("integration_seed: imacals organization not found — skipping");
        return;
    };

    let Some(created_by) = resolve_superuser(pool).await else {
        warn!("integration_seed: no superuser found — skipping (will retry on next start)");
        return;
    };

    // The Log provider goes first so a fresh install always has one working sender: it needs no
    // credentials and writes mail to the API log instead of delivering it. Whichever provider is
    // seeded first becomes the live one (the service only auto-enables when a family has none), so
    // a real provider configured via env below has to be switched on in the dashboard — deliberate,
    // since nobody wants a first boot that starts delivering to real inboxes.
    seed_log(pool, &created_by, &org_id, &domain_id).await;
    seed_smtp(pool, &created_by, &org_id, &domain_id).await;
    seed_mailgun(pool, &created_by, &org_id, &domain_id).await;
    seed_mailchimp(pool, &created_by, &org_id, &domain_id).await;
    seed_google(pool, &created_by, &org_id, &domain_id).await;
    seed_outlook(pool, &created_by, &org_id, &domain_id).await;
    seed_zero_bounce(pool, &created_by, &org_id, &domain_id).await;
}

// ── providers ─────────────────────────────────────────────────────────────────

async fn seed_log(pool: &PgPool, created_by: &Uuid, org_id: &Uuid, domain_id: &Uuid) {
    let from = optional_env("LOG_FROM_EMAIL").unwrap_or_else(|| "no-reply@imacals.local".into());
    create(
        pool,
        created_by,
        org_id,
        domain_id,
        "Dev Log",
        "log-mail",
        IntegrationType::Log,
        vec![inline("LOG_FROM_EMAIL", from, "text", false)],
    )
    .await;
}

// MAIL_* names match docker-compose / .env.example, which point at the Mailpit catcher in dev.
async fn seed_smtp(pool: &PgPool, created_by: &Uuid, org_id: &Uuid, domain_id: &Uuid) {
    let Some(host) = first_env(&["SMTP_HOST", "MAIL_HOST"]) else { return };
    let port = first_env(&["SMTP_PORT", "MAIL_PORT"]).unwrap_or_else(|| "1025".into());
    let from = first_env(&["SMTP_FROM_EMAIL", "MAIL_FROM_ADDRESS"])
        .unwrap_or_else(|| "no-reply@imacals.local".into());

    let mut attributes = vec![
        inline("SMTP_HOST", host, "text", false),
        inline("SMTP_PORT", port, "text", false),
        inline("SMTP_FROM_EMAIL", from, "text", false),
    ];
    if let Some(v) = first_env(&["SMTP_USERNAME", "MAIL_USERNAME"]) {
        attributes.push(inline("SMTP_USERNAME", v, "text", false));
    }
    if let Some(v) = first_env(&["SMTP_PASSWORD", "MAIL_PASSWORD"]) {
        attributes.push(inline("SMTP_PASSWORD", v, "password", true));
    }
    if let Some(v) = first_env(&["SMTP_FROM_NAME", "MAIL_FROM_NAME"]) {
        attributes.push(inline("SMTP_FROM_NAME", v, "text", false));
    }
    if let Some(v) = optional_env("SMTP_USE_TLS") {
        attributes.push(inline("SMTP_USE_TLS", v, "text", false));
    }

    create(pool, created_by, org_id, domain_id, "Primary SMTP", "smtp-relay", IntegrationType::Smtp, attributes)
        .await;
}

async fn seed_mailgun(pool: &PgPool, created_by: &Uuid, org_id: &Uuid, domain_id: &Uuid) {
    let Some(api_key) = optional_env("MAILGUN_API_KEY") else { return };
    let Some(sending_domain) = optional_env("MAILGUN_DOMAIN") else { return };
    let Some(from) = optional_env("MAILGUN_FROM_EMAIL") else { return };

    let mut attributes = vec![
        inline("MAILGUN_API_KEY", api_key, "password", true),
        inline("MAILGUN_DOMAIN", sending_domain, "text", false),
        inline("MAILGUN_FROM_EMAIL", from, "text", false),
    ];
    for (name, kind) in [
        ("MAILGUN_REGION", "text"),
        ("MAILGUN_FROM_NAME", "text"),
        ("MAILGUN_REPLY_TO", "text"),
    ] {
        if let Some(v) = optional_env(name) {
            attributes.push(inline(name, v, kind, false));
        }
    }

    create(pool, created_by, org_id, domain_id, "Mailgun", "mailgun", IntegrationType::Mailgun, attributes)
        .await;
}

async fn seed_mailchimp(pool: &PgPool, created_by: &Uuid, org_id: &Uuid, domain_id: &Uuid) {
    let Some(api_key) = optional_env("MAILCHIMP_API_KEY") else { return };
    let Some(from) = optional_env("MAILCHIMP_FROM_EMAIL") else { return };
    let Some(from_name) = optional_env("MAILCHIMP_FROM_NAME") else { return };

    create(
        pool,
        created_by,
        org_id,
        domain_id,
        "Mailchimp",
        "mailchimp",
        IntegrationType::Mailchimp,
        vec![
            inline("MAILCHIMP_API_KEY", api_key, "password", true),
            inline("MAILCHIMP_FROM_EMAIL", from, "text", false),
            inline("MAILCHIMP_FROM_NAME", from_name, "text", false),
        ],
    )
    .await;
}

async fn seed_google(pool: &PgPool, created_by: &Uuid, org_id: &Uuid, domain_id: &Uuid) {
    let Some(client_id) = optional_env("GOOGLE_CLIENT_ID") else { return };
    let Some(client_secret) = optional_env("GOOGLE_CLIENT_SECRET") else { return };
    let Some(refresh_token) = optional_env("GOOGLE_REFRESH_TOKEN") else { return };
    let Some(from) = optional_env("GOOGLE_FROM_EMAIL") else { return };

    create(
        pool,
        created_by,
        org_id,
        domain_id,
        "Gmail",
        "gmail",
        IntegrationType::Google,
        vec![
            inline("GOOGLE_CLIENT_ID", client_id, "text", false),
            inline("GOOGLE_CLIENT_SECRET", client_secret, "password", true),
            inline("GOOGLE_REFRESH_TOKEN", refresh_token, "password", true),
            inline("GOOGLE_FROM_EMAIL", from, "text", false),
        ],
    )
    .await;
}

async fn seed_outlook(pool: &PgPool, created_by: &Uuid, org_id: &Uuid, domain_id: &Uuid) {
    let Some(client_id) = optional_env("OUTLOOK_CLIENT_ID") else { return };
    let Some(client_secret) = optional_env("OUTLOOK_CLIENT_SECRET") else { return };
    let Some(tenant_id) = optional_env("OUTLOOK_TENANT_ID") else { return };
    let Some(user_id) = optional_env("OUTLOOK_USER_ID") else { return };
    let Some(username) = optional_env("OUTLOOK_USERNAME") else { return };

    let mut attributes = vec![
        inline("OUTLOOK_CLIENT_ID", client_id, "text", false),
        inline("OUTLOOK_CLIENT_SECRET", client_secret, "password", true),
        inline("OUTLOOK_TENANT_ID", tenant_id, "text", false),
        inline("OUTLOOK_USER_ID", user_id, "text", false),
        inline("OUTLOOK_USERNAME", username, "text", false),
    ];
    if let Some(v) = optional_env("OUTLOOK_CLIENT_STATE") {
        attributes.push(inline("OUTLOOK_CLIENT_STATE", v, "password", true));
    }

    create(pool, created_by, org_id, domain_id, "Outlook", "outlook", IntegrationType::Outlook, attributes)
        .await;
}

async fn seed_zero_bounce(pool: &PgPool, created_by: &Uuid, org_id: &Uuid, domain_id: &Uuid) {
    let Some(api_key) = optional_env("ZEROBOUNCE_API_KEY") else { return };
    create(
        pool,
        created_by,
        org_id,
        domain_id,
        "ZeroBounce",
        "zerobounce",
        IntegrationType::ZeroBounce,
        vec![inline("ZEROBOUNCE_API_KEY", api_key, "password", true)],
    )
    .await;
}

// ── helpers ───────────────────────────────────────────────────────────────────

// One shared create path so every provider gets the same skip-if-present + logging behaviour.
// is_enabled is left to the service: the first provider in a family goes live, the rest wait for
// an admin to switch to them.
#[allow(clippy::too_many_arguments)]
async fn create(
    pool: &PgPool,
    created_by: &Uuid,
    org_id: &Uuid,
    domain_id: &Uuid,
    name: &str,
    slug: &str,
    integration_type: IntegrationType,
    attributes: Vec<InlineAttributeSchema>,
) {
    if already_exists(pool, slug).await {
        return;
    }

    let schema = CreateIntegrationSchema {
        organization_id: Some(*org_id),
        domain_id: *domain_id,
        name: name.into(),
        slug: slug.into(),
        integration_type,
        is_enabled: None,
        attributes: Some(attributes),
    };

    match IntegrationService::create(pool, created_by, org_id, &schema, &ENV.app_secret).await {
        Ok(created) => info!(
            "integration_seed: created {} integration '{}' (enabled: {})",
            name, slug, created.is_enabled
        ),
        Err(e) => warn!("integration_seed: {} create failed — {:?}", name, e),
    }
}

fn inline(
    name: &str,
    value: String,
    attr_type: &str,
    encrypted: bool,
) -> InlineAttributeSchema {
    InlineAttributeSchema {
        name: name.into(),
        value: Some(value),
        attribute_type: attr_type.into(),
        is_encrypted: Some(encrypted),
    }
}

// Treats an empty var the same as an absent one — a blank line in .env should not seed a
// credential-less provider that then fails at send time.
fn optional_env(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(v) if !v.trim().is_empty() => Some(v),
        _ => None,
    }
}

// First name that is set, so SMTP_* takes precedence over the MAIL_* names in .env.example.
fn first_env(names: &[&str]) -> Option<String> {
    names.iter().find_map(|n| optional_env(n))
}

async fn already_exists(pool: &PgPool, slug: &str) -> bool {
    sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM integrations WHERE slug = $1 AND deleted_at IS NULL)",
        slug
    )
    .fetch_one(pool)
    .await
    .unwrap_or(Some(false))
    .unwrap_or(false)
}

async fn resolve_domain(pool: &PgPool) -> Option<Uuid> {
    sqlx::query_scalar!(
        "SELECT id FROM domains WHERE slug = 'default-us' AND deleted_at IS NULL LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

async fn resolve_org(pool: &PgPool) -> Option<Uuid> {
    sqlx::query_scalar!(
        "SELECT id FROM organizations WHERE slug = 'imacals' AND deleted_at IS NULL LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

async fn resolve_superuser(pool: &PgPool) -> Option<Uuid> {
    sqlx::query_scalar!(
        "SELECT id FROM users WHERE is_superuser = TRUE AND deleted_at IS NULL LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::integration::IntegrationCategory;
    use crate::repositories::integration_repository::IntegrationRepository;

    // A fresh install must end up with exactly one live sender — the credential-free Log provider —
    // so a campaign can be exercised before any real provider is configured.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn seeding_gives_a_fresh_install_one_live_sender(pool: PgPool) {
        run(&pool).await;

        let live = IntegrationRepository::find_enabled_by_category(&pool, IntegrationCategory::Email)
            .await
            .expect("a fresh install must have a live Email provider");
        assert_eq!(live.slug, "log-mail");
        assert_eq!(live.integration_type, IntegrationType::Log);
    }

    // Running twice must not duplicate rows — seeds run on every boot.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn seeding_twice_is_idempotent(pool: PgPool) {
        run(&pool).await;
        let after_first = IntegrationRepository::index(&pool).await.unwrap().len();
        run(&pool).await;
        let after_second = IntegrationRepository::index(&pool).await.unwrap().len();
        assert_eq!(after_first, after_second);
    }

    // An admin who disables or renames a seeded provider must not have it re-created on next boot.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn seeding_does_not_resurrect_a_disabled_provider(pool: PgPool) {
        run(&pool).await;
        let log = IntegrationRepository::find_by_slug(&pool, "log-mail").await.unwrap();
        IntegrationRepository::set_enabled(&pool, &log.id, false).await.unwrap();

        run(&pool).await;
        let reloaded = IntegrationRepository::find_by_slug(&pool, "log-mail").await.unwrap();
        assert!(!reloaded.is_enabled, "the seed must not re-enable an admin's choice");
    }
}
