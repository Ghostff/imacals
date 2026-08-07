use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use uuid::Uuid;
use crate::AppState;
use crate::models::polygon_neighbor::CreateNeighborSchema;
use crate::models::user::User;
use crate::repositories::polygon_neighbor_repository::PolygonNeighborRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

macro_rules! require_superuser {
    ($user:expr) => {
        if !$user.is_superuser {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    };
}

// Any logged-in user may list neighbor links so other parts of the app can draw the graph.
pub async fn index(user: User, app: Data<AppState>) -> HttpResponse {
    let _ = user;
    match PolygonNeighborRepository::index(&app.pool).await {
        Ok(rows) => JsonResponse::success(rows),
        Err(e)   => JsonResponse::fatal(e, "polygon_neighbor_controller.index failed"),
    }
}

// Links two polygons as neighbors; only superusers may change the topology.
pub async fn create(user: User, app: Data<AppState>, body: Json<CreateNeighborSchema>) -> HttpResponse {
    require_superuser!(user);

    if body.polygon_id == body.neighbor_polygon_id {
        return JsonResponse::error(ErrorBag::Validation {
            field: "neighbor_polygon_id".into(),
            message: "A polygon cannot be its own neighbor".into(),
        });
    }

    match PolygonNeighborRepository::create(&app.pool, &body.polygon_id, &body.neighbor_polygon_id).await {
        Ok(()) => JsonResponse::success(serde_json::json!({ "message": "Neighbor link created" })),
        Err(e) => JsonResponse::fatal(e, "polygon_neighbor_controller.create failed"),
    }
}

// Breaks the neighbor link between two polygons in both directions.
pub async fn delete(user: User, app: Data<AppState>, path: Path<(Uuid, Uuid)>) -> HttpResponse {
    require_superuser!(user);

    let (polygon_id, neighbor_id) = path.into_inner();
    match PolygonNeighborRepository::delete(&app.pool, &polygon_id, &neighbor_id).await {
        Ok(0) => JsonResponse::error(ErrorBag::NotFound("Neighbor link".into())),
        Ok(_) => JsonResponse::success(serde_json::json!({ "message": "Neighbor link removed" })),
        Err(e) => JsonResponse::fatal(e, "polygon_neighbor_controller.delete failed"),
    }
}
