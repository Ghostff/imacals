use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use sqlx::Error;
use uuid::Uuid;
use crate::AppState;
use crate::models::polygon::{AssignPolygonZoneSchema, CreatePolygonSchema, UpdatePolygonSchema};
use crate::models::user::User;
use crate::repositories::polygon_repository::PolygonRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

// Polygons are global admin data — only superusers may write; any authenticated user may read.
macro_rules! require_superuser {
    ($user:expr) => {
        if !$user.is_superuser {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    };
}

pub async fn index(user: User, app: Data<AppState>) -> HttpResponse {
    let _ = user;
    match PolygonRepository::index(&app.pool).await {
        Ok(rows) => JsonResponse::success(rows),
        Err(e)   => JsonResponse::fatal(e, "polygon_controller.index failed"),
    }
}

pub async fn create(user: User, app: Data<AppState>, body: Json<CreatePolygonSchema>) -> HttpResponse {
    require_superuser!(user);

    match PolygonRepository::create(
        &app.pool,
        &user.id,
        &body.coordinates,
        body.city_id.as_ref(),
    ).await {
        Ok(polygon) => JsonResponse::success(polygon),
        Err(e)      => JsonResponse::fatal(e, "polygon_controller.create failed"),
    }
}

pub async fn show(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    let _ = user;
    match PolygonRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(polygon)             => JsonResponse::success(polygon),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Polygon".into())),
        Err(e)                  => JsonResponse::fatal(e, "polygon_controller.show failed"),
    }
}

pub async fn update(user: User, app: Data<AppState>, id: Path<Uuid>, body: Json<UpdatePolygonSchema>) -> HttpResponse {
    require_superuser!(user);

    let coords = match &body.coordinates {
        Some(c) => c,
        None    => return JsonResponse::error(ErrorBag::Validation { field: "coordinates".into(), message: "coordinates is required".into() }),
    };

    match PolygonRepository::update(&app.pool, &id.into_inner(), coords).await {
        Ok(polygon)             => JsonResponse::success(polygon),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Polygon".into())),
        Err(e)                  => JsonResponse::fatal(e, "polygon_controller.update failed"),
    }
}

pub async fn assign_polygon_zone(user: User, app: Data<AppState>, id: Path<Uuid>, body: Json<AssignPolygonZoneSchema>) -> HttpResponse {
    require_superuser!(user);

    match PolygonRepository::assign_polygon_zone(&app.pool, &id.into_inner(), body.polygon_zone_id.as_ref()).await {
        Ok(polygon)             => JsonResponse::success(polygon),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Polygon".into())),
        Err(e)                  => JsonResponse::fatal(e, "polygon_controller.assign_polygon_zone failed"),
    }
}

// Soft-delete: the row is kept so polygon history can be audited or recovered.
pub async fn delete(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    require_superuser!(user);

    match PolygonRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0)  => JsonResponse::error(ErrorBag::NotFound("Polygon".into())),
        Ok(_)  => JsonResponse::success(serde_json::json!({ "message": "Polygon deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "polygon_controller.delete failed"),
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
            email,
            superuser
        )
        .fetch_one(pool)
        .await
        .unwrap();

        format!("Bearer {}", JwtService::create_access_token(id, 60).unwrap())
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn regular_user_cannot_create_polygon(pool: PgPool) {
        let token = make_token(&pool, "regular@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/polygons").route("", web::post().to(create)))
        ).await;

        let req = test::TestRequest::post()
            .uri("/polygons")
            .insert_header(("Authorization", token))
            .set_json(json!({"coordinates": []}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn superuser_can_create_polygon(pool: PgPool) {
        let token = make_token(&pool, "admin@test.com", true).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/polygons").route("", web::post().to(create)))
        ).await;

        let req = test::TestRequest::post()
            .uri("/polygons")
            .insert_header(("Authorization", token))
            .set_json(json!({"coordinates": [{"lat": 25.77, "lng": -80.19}]}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn any_user_can_list_polygons(pool: PgPool) {
        let token = make_token(&pool, "viewer@test.com", false).await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/polygons").route("", web::get().to(index)))
        ).await;

        let req = test::TestRequest::get()
            .uri("/polygons")
            .insert_header(("Authorization", token))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn delete_nonexistent_polygon_returns_404(pool: PgPool) {
        let token = make_token(&pool, "admin2@test.com", true).await;
        let fake_id = uuid::Uuid::new_v4();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/polygons").route("/{id}", web::delete().to(delete)))
        ).await;

        let req = test::TestRequest::delete()
            .uri(&format!("/polygons/{}", fake_id))
            .insert_header(("Authorization", token))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
