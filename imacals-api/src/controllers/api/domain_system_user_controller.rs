use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde_json::json;
use uuid::Uuid;

use crate::AppState;
use crate::models::domain_system_user::CreateDomainSystemUserSchema;
use crate::models::user::User;
use crate::repositories::domain_system_user_repository::DomainSystemUserRepository;
use crate::repositories::organization_user_role_repository::OrganizationUserRoleRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

pub async fn index(user: User, app: Data<AppState>) -> HttpResponse {
    let _ = user;
    match DomainSystemUserRepository::index(&app.pool).await {
        Ok(rows) => JsonResponse::success(rows),
        Err(e)   => JsonResponse::fatal(e, "domain_system_user_controller.index failed"),
    }
}

// Returns the job-title roles that may be used as system users (broker, realtor, hml, insurance).
pub async fn eligible_roles(user: User, app: Data<AppState>) -> HttpResponse {
    let _ = user;
    match OrganizationUserRoleRepository::system_user_eligible(&app.pool).await {
        Ok(roles) => JsonResponse::success(roles),
        Err(e)    => JsonResponse::fatal(e, "domain_system_user_controller.eligible_roles failed"),
    }
}

pub async fn upsert(
    user: User,
    app: Data<AppState>,
    body: Json<CreateDomainSystemUserSchema>,
) -> HttpResponse {
    if !user.is_superuser {
        return JsonResponse::error(ErrorBag::Forbidden);
    }

    // Reject if the chosen role is not system_user_eligible.
    let eligible = match OrganizationUserRoleRepository::system_user_eligible(&app.pool).await {
        Ok(roles) => roles,
        Err(e)    => return JsonResponse::fatal(e, "domain_system_user_controller.upsert failed"),
    };
    if !eligible.iter().any(|r| r.id == body.user_role_id) {
        return JsonResponse::error(ErrorBag::Validation {
            field:   "user_role_id".into(),
            message: "role is not eligible for system user assignment".into(),
        });
    }

    match DomainSystemUserRepository::upsert(&app.pool, &body, &user.id).await {
        Ok(record) => JsonResponse::success(record),
        Err(e)     => JsonResponse::fatal(e, "domain_system_user_controller.upsert failed"),
    }
}

pub async fn delete(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    if !user.is_superuser {
        return JsonResponse::error(ErrorBag::Forbidden);
    }

    match DomainSystemUserRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0)  => JsonResponse::error(ErrorBag::NotFound("SystemUser".into())),
        Ok(_)  => JsonResponse::success(json!({ "message": "System user removed successfully" })),
        Err(e) => JsonResponse::fatal(e, "domain_system_user_controller.delete failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use actix_web::http::StatusCode;
    use sqlx::PgPool;
    use serde_json::json;
    use crate::AppState;
    use crate::services::jwt_service::JwtService;

    async fn make_token(pool: &PgPool, email: &str, superuser: bool) -> String {
        let id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, is_superuser, current_logged_in_at)
             VALUES ('T','T',$1,'x',$2,NOW()) RETURNING id",
            email, superuser
        ).fetch_one(pool).await.unwrap();
        format!("Bearer {}", JwtService::create_access_token(id, 60).unwrap())
    }

    async fn default_domain(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!("SELECT id FROM domains WHERE slug = 'default-us' LIMIT 1")
            .fetch_one(pool).await.unwrap()
    }

    async fn broker_role_id(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!(
            "SELECT id FROM organization_user_role
             WHERE name = 'broker' AND system_user_eligible = TRUE AND organization_id IS NULL LIMIT 1"
        ).fetch_one(pool).await.unwrap()
    }

    async fn ineligible_role_id(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!(
            "SELECT id FROM organization_user_role
             WHERE system_user_eligible = FALSE AND organization_id IS NULL LIMIT 1"
        ).fetch_one(pool).await.unwrap()
    }

    fn app_scope(pool: PgPool) -> actix_web::App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new()
            .app_data(web::Data::new(AppState { pool }))
            .service(
                web::scope("/domain-system-users")
                    .route("",                  web::get().to(index))
                    .route("/eligible-roles",   web::get().to(eligible_roles))
                    .route("",                  web::post().to(upsert))
                    .route("/{id}",             web::delete().to(delete)),
            )
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn authenticated_user_can_list(pool: PgPool) {
        let token = make_token(&pool, "list_dsu@test.com", false).await;
        let app   = test::init_service(app_scope(pool)).await;
        let req   = test::TestRequest::get()
            .uri("/domain-system-users")
            .insert_header(("Authorization", token)).to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn unauthenticated_list_is_rejected(pool: PgPool) {
        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::get().uri("/domain-system-users").to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn eligible_roles_returns_ok(pool: PgPool) {
        let token = make_token(&pool, "elroles@test.com", false).await;
        // Clone: the pool is still needed below to mint a second token.
        let app   = test::init_service(app_scope(pool.clone())).await;
        let req   = test::TestRequest::get()
            .uri("/domain-system-users/eligible-roles")
            .insert_header(("Authorization", token)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::call_and_read_body_json(&app,
            test::TestRequest::get()
                .uri("/domain-system-users/eligible-roles")
                .insert_header(("Authorization", make_token(&pool, "elroles2@test.com", false).await))
                .to_request()
        ).await;
        let names: Vec<&str> = body["data"].as_array().unwrap()
            .iter().filter_map(|r| r["name"].as_str()).collect();
        assert!(names.contains(&"broker"),  "broker must be eligible");
        assert!(names.contains(&"realtor"), "realtor must be eligible");
        assert!(names.contains(&"hml"),     "hml must be eligible");
        assert!(names.contains(&"insurance"), "insurance must be eligible");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn non_superuser_cannot_upsert(pool: PgPool) {
        let token     = make_token(&pool, "nsuper_dsu@test.com", false).await;
        let domain_id = default_domain(&pool).await;
        let role_id   = broker_role_id(&pool).await;
        let user_id   = sqlx::query_scalar!(
            "INSERT INTO users (first_name,last_name,email,password,current_logged_in_at)
             VALUES ('A','B','target_dsu@test.com','x',NOW()) RETURNING id"
        ).fetch_one(&pool).await.unwrap();

        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::post()
            .uri("/domain-system-users")
            .insert_header(("Authorization", token))
            .set_json(json!({ "domain_id": domain_id, "user_id": user_id, "user_role_id": role_id }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_upsert_eligible_role(pool: PgPool) {
        let token     = make_token(&pool, "super_dsu@test.com", true).await;
        let domain_id = default_domain(&pool).await;
        let role_id   = broker_role_id(&pool).await;
        let user_id   = sqlx::query_scalar!(
            "INSERT INTO users (first_name,last_name,email,password,current_logged_in_at)
             VALUES ('A','B','target2_dsu@test.com','x',NOW()) RETURNING id"
        ).fetch_one(&pool).await.unwrap();

        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::post()
            .uri("/domain-system-users")
            .insert_header(("Authorization", token))
            .set_json(json!({ "domain_id": domain_id, "user_id": user_id, "user_role_id": role_id }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn ineligible_role_is_rejected(pool: PgPool) {
        let token     = make_token(&pool, "super_inel@test.com", true).await;
        let domain_id = default_domain(&pool).await;
        let role_id   = ineligible_role_id(&pool).await;
        let user_id   = sqlx::query_scalar!(
            "INSERT INTO users (first_name,last_name,email,password,current_logged_in_at)
             VALUES ('A','B','target3_dsu@test.com','x',NOW()) RETURNING id"
        ).fetch_one(&pool).await.unwrap();

        let app = test::init_service(app_scope(pool)).await;
        let req = test::TestRequest::post()
            .uri("/domain-system-users")
            .insert_header(("Authorization", token))
            .set_json(json!({ "domain_id": domain_id, "user_id": user_id, "user_role_id": role_id }))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
