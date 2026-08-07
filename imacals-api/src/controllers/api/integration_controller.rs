use actix_web::web::{Data, Json, Path, Query};
use actix_web::HttpResponse;
use serde::Deserialize;
use serde_json::json;
use sqlx::Error;
use uuid::Uuid;

use crate::config::ENV;
use crate::AppState;
use crate::models::integration::{
    CreateIntegrationSchema, IntegrationCategory, IntegrationType, SetEnabledSchema,
    UpdateIntegrationSchema,
};
use crate::models::user::User;
use crate::repositories::attribute_repository::AttributeRepository;
use crate::repositories::integration_repository::IntegrationRepository;
use crate::repositories::organization_repository::OrganizationRepository;
use crate::services::integration_service::IntegrationService;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::integration_type_defs;
use crate::utilities::json_response::JsonResponse;

// Integrations are platform-level config — reads are open to any authenticated user;
// writes are restricted to superusers to prevent credential tampering.
macro_rules! require_superuser {
    ($user:expr) => {
        if !$user.is_superuser {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    };
}

// ?category=email narrows the listing to one provider family — how the dashboard renders its
// Email / Verification sections without filtering client-side.
#[derive(Debug, Deserialize)]
pub struct IndexQuery {
    pub category: Option<IntegrationCategory>,
}

pub async fn index(_user: User, app: Data<AppState>, query: Query<IndexQuery>) -> HttpResponse {
    let result = match query.category {
        Some(category) => IntegrationRepository::index_for_category(&app.pool, category).await,
        None => IntegrationRepository::index(&app.pool).await,
    };
    match result {
        Ok(rows) => JsonResponse::success(rows),
        Err(e)   => JsonResponse::fatal(e, "integration_controller.index failed"),
    }
}

// The credential template every provider type expects, so the dashboard renders its form from the
// backend's definition instead of keeping a second copy of the field list in TypeScript.
pub async fn provider_types(_user: User) -> HttpResponse {
    let types = [
        IntegrationType::Smtp,
        IntegrationType::Log,
        IntegrationType::Mailgun,
        IntegrationType::Mailchimp,
        IntegrationType::Google,
        IntegrationType::Outlook,
        IntegrationType::ZeroBounce,
        IntegrationType::Custom,
    ];

    let payload: Vec<_> = types
        .iter()
        .map(|t| {
            json!({
                "integration_type":     t,
                "integration_category": t.category(),
                "fields": integration_type_defs::fields_for_type(t).iter().map(|f| json!({
                    "name":         f.name,
                    "label":        f.label,
                    "type":         f.field_type,
                    "is_encrypted": f.is_encrypted,
                    "is_required":  f.is_required,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    JsonResponse::success(payload)
}

pub async fn show(_user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    match IntegrationRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(integration)         => JsonResponse::success(integration),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Integration".into())),
        Err(e)                  => JsonResponse::fatal(e, "integration_controller.show failed"),
    }
}

pub async fn create(
    user: User,
    app: Data<AppState>,
    body: Json<CreateIntegrationSchema>,
) -> HttpResponse {
    require_superuser!(user);

    // When no organization is specified, default to the "imacals" platform org.
    let org_id = match body.organization_id {
        Some(id) => id,
        None => match OrganizationRepository::find_by_slug(&app.pool, "imacals").await {
            Ok(org) => org.id,
            Err(_)  => return JsonResponse::error(
                ErrorBag::InternalServerError("Could not resolve default organization".into())
            ),
        },
    };

    match IntegrationService::create(&app.pool, &user.id, &org_id, &body, &ENV.app_secret).await {
        Ok(integration) => JsonResponse::success(integration),
        Err(e)          => JsonResponse::error(e),
    }
}

pub async fn update(
    user: User,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<UpdateIntegrationSchema>,
) -> HttpResponse {
    require_superuser!(user);
    match IntegrationRepository::update(&app.pool, &id.into_inner(), &body).await {
        Ok(integration)         => JsonResponse::success(integration),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Integration".into())),
        Err(e)                  => JsonResponse::fatal(e, "integration_controller.update failed"),
    }
}

pub async fn delete(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    require_superuser!(user);
    match IntegrationRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0) => JsonResponse::error(ErrorBag::NotFound("Integration".into())),
        Ok(_) => JsonResponse::success(json!({ "message": "Integration deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "integration_controller.delete failed"),
    }
}

// Switches which provider of a family is live. Separate from `update` because enabling one
// provider disables its siblings — a plain field patch can't express that.
pub async fn set_enabled(
    user: User,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<SetEnabledSchema>,
) -> HttpResponse {
    require_superuser!(user);
    match IntegrationService::set_enabled(&app.pool, &id.into_inner(), body.is_enabled).await {
        Ok(integration) => JsonResponse::success(integration),
        Err(e)          => JsonResponse::error(e),
    }
}

// Returns all active attributes for this integration, with encrypted values withheld: ciphertext
// is useless to the UI and shipping it to the browser only widens the blast radius of a leak.
// Encrypted fields can be overwritten, never read back.
pub async fn attributes(_user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    match AttributeRepository::find_for_owner(&app.pool, "integrations", &id.into_inner()).await {
        Ok(attrs) => {
            let masked: Vec<_> = attrs
                .into_iter()
                .map(|mut attr| {
                    if attr.is_encrypted {
                        attr.value = None;
                    }
                    attr
                })
                .collect();
            JsonResponse::success(masked)
        }
        Err(e) => JsonResponse::fatal(e, "integration_controller.attributes failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use serde_json::json;
    use sqlx::PgPool;

    use crate::services::jwt_service::JwtService;
    use crate::AppState;

    async fn make_token(pool: &PgPool, email: &str, superuser: bool) -> String {
        let id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T',$1,'x',$2,NOW()) RETURNING id",
            email,
            superuser
        )
        .fetch_one(pool)
        .await
        .unwrap();
        format!("Bearer {}", JwtService::create_access_token(id, 60).unwrap())
    }

    async fn default_domain_id(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!("SELECT id FROM domains WHERE slug = 'default-us' LIMIT 1")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn make_integration(pool: &PgPool, user_id: Uuid, domain_id: Uuid, slug: &str) -> Uuid {
        let org_id = sqlx::query_scalar!(
            "SELECT id FROM organizations WHERE slug = 'imacals' LIMIT 1"
        )
        .fetch_one(pool)
        .await
        .unwrap();
        sqlx::query_scalar!(
            "INSERT INTO integrations (organization_id, domain_id, created_by, name, slug, integration_type)
             VALUES ($1, $2, $3, $4, $5, 'custom') RETURNING id",
            org_id,
            domain_id,
            user_id,
            slug,
            slug,
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn any_user_can_list_integrations(pool: PgPool) {
        let token = make_token(&pool, "viewer_int@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("", web::get().to(index))),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/integrations")
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn regular_user_cannot_create_integration(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let token = make_token(&pool, "regular_int@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("", web::post().to(create))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/integrations")
            .insert_header(("Authorization", token))
            .set_json(json!({ "domain_id": domain_id, "name": "Relay", "slug": "relay", "integration_type": "smtp" }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_create_custom_integration(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let token = make_token(&pool, "admin_int_create@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("", web::post().to(create))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/integrations")
            .insert_header(("Authorization", token))
            .set_json(json!({
                "domain_id":        domain_id,
                "name":             "Custom Integration",
                "slug":             "custom-int-ctrl",
                "integration_type": "custom"
            }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_create_smtp_integration_with_attributes(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let token = make_token(&pool, "admin_smtp_create@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("", web::post().to(create))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/integrations")
            .insert_header(("Authorization", token))
            .set_json(json!({
                "domain_id":        domain_id,
                "name":             "Campaign Relay",
                "slug":             "smtp-ctrl-test",
                "integration_type": "smtp",
                "attributes": [
                    { "name": "SMTP_HOST",       "value": "imacals-mail",          "type": "text",     "is_encrypted": false },
                    { "name": "SMTP_PORT",       "value": "1025",                "type": "text",     "is_encrypted": false },
                    { "name": "SMTP_FROM_EMAIL", "value": "no-reply@imacals.local","type": "text",     "is_encrypted": false },
                    { "name": "SMTP_PASSWORD",   "value": "secret",              "type": "password", "is_encrypted": true  }
                ]
            }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_fails_when_required_field_missing(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let token = make_token(&pool, "admin_mailgun_missing@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("", web::post().to(create))),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/integrations")
            .insert_header(("Authorization", token))
            // Missing MAILGUN_DOMAIN and MAILGUN_FROM_EMAIL
            .set_json(json!({
                "domain_id":        domain_id,
                "name":             "Incomplete Mailgun",
                "slug":             "incomplete-mailgun",
                "integration_type": "mailgun",
                "attributes": [
                    { "name": "MAILGUN_API_KEY", "value": "key-123", "type": "password", "is_encrypted": true }
                ]
            }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_show_integration(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T','show_int@test.com','x',true,NOW()) RETURNING id"
        ).fetch_one(&pool).await.unwrap();
        let token = format!("Bearer {}", JwtService::create_access_token(user_id, 60).unwrap());
        let int_id = make_integration(&pool, user_id, domain_id, "show-int").await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("/{id}", web::get().to(show))),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/integrations/{}", int_id))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn show_nonexistent_integration_returns_404(pool: PgPool) {
        let token = make_token(&pool, "viewer_int_404@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("/{id}", web::get().to(show))),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/integrations/{}", Uuid::new_v4()))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_delete_integration(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T','del_int@test.com','x',true,NOW()) RETURNING id"
        ).fetch_one(&pool).await.unwrap();
        let token = format!("Bearer {}", JwtService::create_access_token(user_id, 60).unwrap());
        let int_id = make_integration(&pool, user_id, domain_id, "del-int-ctrl").await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("/{id}", web::delete().to(delete))),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/integrations/{}", int_id))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn delete_nonexistent_integration_returns_404(pool: PgPool) {
        let token = make_token(&pool, "admin_int_del2@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("/{id}", web::delete().to(delete))),
        )
        .await;
        let req = test::TestRequest::delete()
            .uri(&format!("/integrations/{}", Uuid::new_v4()))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn any_user_can_list_attributes_for_integration(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T','attrs_list@test.com','x',true,NOW()) RETURNING id"
        ).fetch_one(&pool).await.unwrap();
        let viewer_token = format!(
            "Bearer {}",
            JwtService::create_access_token(
                sqlx::query_scalar!(
                    "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
                     VALUES ('T','T','attrs_viewer@test.com','x',false,NOW()) RETURNING id"
                ).fetch_one(&pool).await.unwrap(),
                60
            ).unwrap()
        );
        let int_id = make_integration(&pool, user_id, domain_id, "attrs-list-int").await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations")
                    .route("/{id}/attributes", web::get().to(attributes))),
        )
        .await;
        let req = test::TestRequest::get()
            .uri(&format!("/integrations/{}/attributes", int_id))
            .insert_header(("Authorization", viewer_token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    // Ciphertext must never leave the API — the dashboard overwrites secrets, it never reads them.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn encrypted_attribute_values_are_withheld(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let token = make_token(&pool, "admin_mask@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(
                    web::scope("/integrations")
                        .route("", web::post().to(create))
                        .route("/{id}/attributes", web::get().to(attributes)),
                ),
        )
        .await;

        let create_req = test::TestRequest::post()
            .uri("/integrations")
            .insert_header(("Authorization", token.clone()))
            .set_json(json!({
                "domain_id":        domain_id,
                "name":             "Masked Relay",
                "slug":             "masked-relay",
                "integration_type": "smtp",
                "attributes": [
                    { "name": "SMTP_HOST",       "value": "imacals-mail",           "type": "text",     "is_encrypted": false },
                    { "name": "SMTP_PORT",       "value": "1025",                 "type": "text",     "is_encrypted": false },
                    { "name": "SMTP_FROM_EMAIL", "value": "no-reply@imacals.local", "type": "text",     "is_encrypted": false },
                    { "name": "SMTP_PASSWORD",   "value": "s3cret",               "type": "password", "is_encrypted": true  }
                ]
            }))
            .to_request();
        let created: serde_json::Value =
            test::call_and_read_body_json(&app, create_req).await;
        let id = created["data"]["id"].as_str().unwrap().to_string();

        let req = test::TestRequest::get()
            .uri(&format!("/integrations/{id}/attributes"))
            .insert_header(("Authorization", token))
            .to_request();
        let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        let rows = body["data"].as_array().unwrap();

        let password = rows.iter().find(|a| a["name"] == "SMTP_PASSWORD").unwrap();
        assert!(password["value"].is_null(), "encrypted value must not be returned");
        let host = rows.iter().find(|a| a["name"] == "SMTP_HOST").unwrap();
        assert_eq!(host["value"], "imacals-mail", "plaintext config is still readable");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn index_can_be_filtered_by_category(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let token = make_token(&pool, "admin_catfilter@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(
                    web::scope("/integrations")
                        .route("", web::get().to(index))
                        .route("", web::post().to(create)),
                ),
        )
        .await;

        for (slug, kind, attrs) in [
            (
                "cat-relay",
                "smtp",
                json!([
                    { "name": "SMTP_HOST",       "value": "imacals-mail",           "type": "text", "is_encrypted": false },
                    { "name": "SMTP_PORT",       "value": "1025",                 "type": "text", "is_encrypted": false },
                    { "name": "SMTP_FROM_EMAIL", "value": "no-reply@imacals.local", "type": "text", "is_encrypted": false }
                ]),
            ),
            (
                "cat-zb",
                "zero-bounce",
                json!([
                    { "name": "ZEROBOUNCE_API_KEY", "value": "zb-key", "type": "password", "is_encrypted": true }
                ]),
            ),
        ] {
            let req = test::TestRequest::post()
                .uri("/integrations")
                .insert_header(("Authorization", token.clone()))
                .set_json(json!({
                    "domain_id":        domain_id,
                    "name":             slug,
                    "slug":             slug,
                    "integration_type": kind,
                    "attributes":       attrs
                }))
                .to_request();
            assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK, "{slug}");
        }

        let req = test::TestRequest::get()
            .uri("/integrations?category=email")
            .insert_header(("Authorization", token))
            .to_request();
        let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        let rows = body["data"].as_array().unwrap();
        assert!(rows.iter().any(|r| r["slug"] == "cat-relay"));
        assert!(
            !rows.iter().any(|r| r["slug"] == "cat-zb"),
            "a verifier must not appear in the Email family"
        );
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn regular_user_cannot_switch_the_live_provider(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T','switch_owner@test.com','x',true,NOW()) RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let int_id = make_integration(&pool, user_id, domain_id, "switch-target").await;
        let token = make_token(&pool, "switch_regular@test.com", false).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("/{id}/enabled", web::put().to(set_enabled))),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/integrations/{int_id}/enabled"))
            .insert_header(("Authorization", token))
            .set_json(json!({ "is_enabled": false }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_switch_the_live_provider(pool: PgPool) {
        let domain_id = default_domain_id(&pool).await;
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T','switch_admin@test.com','x',true,NOW()) RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let token = format!("Bearer {}", JwtService::create_access_token(user_id, 60).unwrap());
        let int_id = make_integration(&pool, user_id, domain_id, "switch-admin-target").await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("/{id}/enabled", web::put().to(set_enabled))),
        )
        .await;
        let req = test::TestRequest::put()
            .uri(&format!("/integrations/{int_id}/enabled"))
            .insert_header(("Authorization", token))
            .set_json(json!({ "is_enabled": false }))
            .to_request();
        let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(body["data"]["is_enabled"], false);
    }

    // The dashboard renders its credential forms from this, so every type must come back with the
    // category and field list attached.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn provider_types_returns_field_templates(pool: PgPool) {
        let token = make_token(&pool, "types_viewer@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/integrations").route("/provider-types", web::get().to(provider_types))),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/integrations/provider-types")
            .insert_header(("Authorization", token))
            .to_request();
        let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        let rows = body["data"].as_array().unwrap();

        let smtp = rows.iter().find(|r| r["integration_type"] == "smtp").unwrap();
        assert_eq!(smtp["integration_category"], "email");
        assert!(smtp["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["name"] == "SMTP_HOST" && f["is_required"] == true));

        let zb = rows.iter().find(|r| r["integration_type"] == "zero-bounce").unwrap();
        assert_eq!(zb["integration_category"], "email-validation");
    }
}
