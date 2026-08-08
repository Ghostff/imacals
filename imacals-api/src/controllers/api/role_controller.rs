use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse};
use crate::AppState;
use crate::models::user::User;
use crate::repositories::role_repository::RoleRepository;
use crate::utilities::http_request::QueryRequestExt;

pub async fn index(req: HttpRequest, user: User, app: Data<AppState>) -> HttpResponse {
    crate::gate!(&app.pool, &user, "roles.view");

    if req.get_query("include-permission").is_some() {
        return match RoleRepository::index_with_permissions(&app.pool).await {
            Ok(roles) => JsonResponse::success(roles),
            Err(e) => JsonResponse::fatal(e, "role_controller.index failed"),
        }
    }

    match RoleRepository::index(&app.pool).await {
        Ok(roles) => JsonResponse::success(roles),
        Err(e) => JsonResponse::fatal(e, "role_controller.index failed"),
    }
}
