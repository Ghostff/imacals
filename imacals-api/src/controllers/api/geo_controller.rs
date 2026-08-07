use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use uuid::Uuid;
use crate::AppState;
use crate::models::user::User;
use crate::repositories::geo_repository::GeoRepository;
use crate::utilities::json_response::JsonResponse;

pub async fn countries(user: User, app: Data<AppState>) -> HttpResponse {
    let _ = user;
    match GeoRepository::list_countries(&app.pool).await {
        Ok(rows) => JsonResponse::success(rows),
        Err(e)   => JsonResponse::fatal(e, "geo_controller.countries failed"),
    }
}

pub async fn states(user: User, app: Data<AppState>, country_id: Path<Uuid>) -> HttpResponse {
    let _ = user;
    match GeoRepository::list_states_by_country(&app.pool, &country_id.into_inner()).await {
        Ok(rows) => JsonResponse::success(rows),
        Err(e)   => JsonResponse::fatal(e, "geo_controller.states failed"),
    }
}

pub async fn cities(user: User, app: Data<AppState>, state_id: Path<Uuid>) -> HttpResponse {
    let _ = user;
    match GeoRepository::list_cities_by_state(&app.pool, &state_id.into_inner()).await {
        Ok(rows) => JsonResponse::success(rows),
        Err(e)   => JsonResponse::fatal(e, "geo_controller.cities failed"),
    }
}
