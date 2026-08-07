use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use sqlx::Error;
use uuid::Uuid;
use validator::Validate;
use crate::AppState;
use crate::models::polygon_zone::{CreatePolygonZoneSchema, UpdatePolygonZoneSchema};
use crate::models::user::User;
use crate::repositories::polygon_zone_repository::PolygonZoneRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

// Polygon zones are global admin data — only superusers may write; any authenticated user may read.
macro_rules! require_superuser {
    ($user:expr) => {
        if !$user.is_superuser {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    };
}

pub async fn index(user: User, app: Data<AppState>) -> HttpResponse {
    let _ = user;
    match PolygonZoneRepository::index(&app.pool).await {
        Ok(zones) => JsonResponse::success(zones),
        Err(e)    => JsonResponse::fatal(e, "polygon_zone_controller.index failed"),
    }
}

pub async fn create(user: User, app: Data<AppState>, body: Json<CreatePolygonZoneSchema>) -> HttpResponse {
    require_superuser!(user);

    if let Err(errors) = body.validate() {
        let first = errors.field_errors().into_iter().next();
        if let Some((field, messages)) = first {
            let message = messages.first().and_then(|e| e.message.as_ref()).map(|m| m.to_string()).unwrap_or_default();
            return JsonResponse::error(ErrorBag::Validation { field: field.to_string(), message });
        }
    }

    match PolygonZoneRepository::create(&app.pool, &body, &user.id).await {
        Ok(zone) => JsonResponse::success(zone),
        Err(e)   => JsonResponse::fatal(e, "polygon_zone_controller.create failed"),
    }
}

pub async fn update(user: User, app: Data<AppState>, id: Path<Uuid>, body: Json<UpdatePolygonZoneSchema>) -> HttpResponse {
    require_superuser!(user);

    if let Err(errors) = body.validate() {
        let first = errors.field_errors().into_iter().next();
        if let Some((field, messages)) = first {
            let message = messages.first().and_then(|e| e.message.as_ref()).map(|m| m.to_string()).unwrap_or_default();
            return JsonResponse::error(ErrorBag::Validation { field: field.to_string(), message });
        }
    }

    match PolygonZoneRepository::update(&app.pool, &id.into_inner(), &body).await {
        Ok(zone)                => JsonResponse::success(zone),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("PolygonZone".into())),
        Err(e)                  => JsonResponse::fatal(e, "polygon_zone_controller.update failed"),
    }
}
