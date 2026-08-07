use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde_json::json;
use sqlx::Error;
use uuid::Uuid;

use crate::AppState;
use crate::models::domain::CreateDomainSchema;
use crate::models::user::User;
use crate::repositories::domain_repository::DomainRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

macro_rules! require_superuser {
    ($user:expr) => {
        if !$user.is_superuser {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    };
}

pub async fn index(user: User, app: Data<AppState>) -> HttpResponse {
    let _ = user;
    match DomainRepository::index(&app.pool).await {
        Ok(rows) => JsonResponse::success(rows),
        Err(e)   => JsonResponse::fatal(e, "domain_controller.index failed"),
    }
}

pub async fn show(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    let _ = user;
    match DomainRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(d)                   => JsonResponse::success(d),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Domain".into())),
        Err(e)                  => JsonResponse::fatal(e, "domain_controller.show failed"),
    }
}

pub async fn create(user: User, app: Data<AppState>, body: Json<CreateDomainSchema>) -> HttpResponse {
    require_superuser!(user);
    match DomainRepository::create(&app.pool, &body).await {
        Ok(d)  => JsonResponse::success(d),
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                match db_err.constraint() {
                    Some("domains_slug_unique") => return JsonResponse::error(ErrorBag::Validation {
                        field: "slug".into(), message: "Slug already taken".into(),
                    }),
                    Some("domains_location_unique") => return JsonResponse::error(ErrorBag::Validation {
                        field: "location".into(),
                        message: "A domain with this country/state/city combination already exists".into(),
                    }),
                    _ => {}
                }
            }
            JsonResponse::fatal(e, "domain_controller.create failed")
        }
    }
}

pub async fn update(user: User, app: Data<AppState>, id: Path<Uuid>, body: Json<CreateDomainSchema>) -> HttpResponse {
    require_superuser!(user);
    match DomainRepository::update(&app.pool, &id.into_inner(), &body).await {
        Ok(d)                   => JsonResponse::success(d),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Domain".into())),
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                match db_err.constraint() {
                    Some("domains_slug_unique") => return JsonResponse::error(ErrorBag::Validation {
                        field: "slug".into(), message: "Slug already taken".into(),
                    }),
                    Some("domains_location_unique") => return JsonResponse::error(ErrorBag::Validation {
                        field: "location".into(),
                        message: "A domain with this country/state/city combination already exists".into(),
                    }),
                    _ => {}
                }
            }
            JsonResponse::fatal(e, "domain_controller.update failed")
        }
    }
}

pub async fn delete(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    require_superuser!(user);
    match DomainRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0)  => JsonResponse::error(ErrorBag::NotFound("Domain".into())),
        Ok(_)  => JsonResponse::success(json!({ "message": "Domain deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "domain_controller.delete failed"),
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


    // domains_location_unique is UNIQUE NULLS NOT DISTINCT over (country, state, city), and the
    // migrations already seed the country-level US row — so a test that creates its own domain has
    // to claim a location nobody holds yet.
    async fn free_us_location(pool: &PgPool) -> (Uuid, Uuid) {
        let country_id = sqlx::query_scalar!("SELECT id FROM countries WHERE iso3_code = 'USA' LIMIT 1")
            .fetch_one(pool).await.unwrap();
        let state_id = sqlx::query_scalar!(
            "SELECT s.id FROM states s
             WHERE s.country_id = $1
               AND NOT EXISTS (
                   SELECT 1 FROM domains d
                   WHERE d.state_id = s.id AND d.city_id IS NULL AND d.deleted_at IS NULL
               )
             ORDER BY s.name
             LIMIT 1",
            country_id
        ).fetch_one(pool).await.unwrap();
        (country_id, state_id)
    }

    // Returns App<T> rather than `impl ServiceFactory<..>`: App itself is only an
    // IntoServiceFactory, so the latter never satisfies test::init_service.
    fn make_app_service(pool: sqlx::PgPool) -> App<
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
                web::scope("/domains")
                    .route("",      web::get().to(index))
                    .route("",      web::post().to(create))
                    .route("/{id}", web::get().to(show))
                    .route("/{id}", web::put().to(update))
                    .route("/{id}", web::delete().to(delete))
            )
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn authenticated_user_can_list_domains(pool: PgPool) {
        let token = make_token(&pool, "viewer@test.com", false).await;
        let app   = test::init_service(make_app_service(pool)).await;
        let req   = test::TestRequest::get().uri("/domains")
            .insert_header(("Authorization", token)).to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn unauthenticated_request_is_rejected(pool: PgPool) {
        let app = test::init_service(make_app_service(pool)).await;
        let req = test::TestRequest::get().uri("/domains").to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn response_includes_seeded_domain(pool: PgPool) {
        let token = make_token(&pool, "viewer2@test.com", false).await;
        let app   = test::init_service(make_app_service(pool)).await;
        let req   = test::TestRequest::get().uri("/domains")
            .insert_header(("Authorization", token)).to_request();
        let body: serde_json::Value = test::call_and_read_body_json(&app, req).await;
        let slugs: Vec<&str> = body["data"].as_array().unwrap()
            .iter().filter_map(|d| d["slug"].as_str()).collect();
        assert!(slugs.contains(&"default-us"), "seeded default-us domain must appear");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn regular_user_cannot_create_domain(pool: PgPool) {
        let token      = make_token(&pool, "regular@test.com", false).await;
        let country_id = sqlx::query_scalar!("SELECT id FROM countries WHERE iso3_code = 'USA' LIMIT 1")
            .fetch_one(&pool).await.unwrap();
        let app = test::init_service(make_app_service(pool)).await;
        let req = test::TestRequest::post().uri("/domains")
            .insert_header(("Authorization", token))
            .set_json(json!({"name":"X","slug":"x","country_id": country_id}))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_create_domain(pool: PgPool) {
        let token = make_token(&pool, "admin@test.com", true).await;
        let (country_id, state_id) = free_us_location(&pool).await;
        let app = test::init_service(make_app_service(pool)).await;
        let req = test::TestRequest::post().uri("/domains")
            .insert_header(("Authorization", token))
            .set_json(json!({"name":"New Region","slug":"new-region","country_id": country_id, "state_id": state_id}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn duplicate_slug_returns_validation_error(pool: PgPool) {
        let token = make_token(&pool, "admin2@test.com", true).await;
        let (country_id, taken_state) = free_us_location(&pool).await;
        let app = test::init_service(make_app_service(pool.clone())).await;

        // First create succeeds; second with the same slug must fail.
        sqlx::query!(
            "INSERT INTO domains (name, slug, country_id, state_id) VALUES ('X','dup-slug',$1,$2)",
            country_id, taken_state
        ).execute(&pool).await.unwrap();

        // A different location, so the only thing wrong with the second request is the slug.
        let (_, free_state) = free_us_location(&pool).await;
        let req = test::TestRequest::post().uri("/domains")
            .insert_header(("Authorization", token))
            .set_json(json!({"name":"Y","slug":"dup-slug","country_id": country_id, "state_id": free_state}))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_delete_domain(pool: PgPool) {
        let token = make_token(&pool, "del@test.com", true).await;
        let (country_id, state_id) = free_us_location(&pool).await;
        let domain_id  = sqlx::query_scalar!(
            "INSERT INTO domains (name, slug, country_id, state_id) VALUES ('Del','del-test',$1,$2) RETURNING id",
            country_id, state_id
        ).fetch_one(&pool).await.unwrap();

        let app = test::init_service(make_app_service(pool)).await;
        let req = test::TestRequest::delete()
            .uri(&format!("/domains/{domain_id}"))
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn delete_nonexistent_domain_returns_not_found(pool: PgPool) {
        let token = make_token(&pool, "del2@test.com", true).await;
        let app   = test::init_service(make_app_service(pool)).await;
        let req   = test::TestRequest::delete()
            .uri("/domains/00000000-0000-0000-0000-000000000000")
            .insert_header(("Authorization", token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }
}
