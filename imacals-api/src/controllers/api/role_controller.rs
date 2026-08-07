use actix_web::web::{Data};
use actix_web::{HttpRequest, HttpResponse};
use crate::AppState;
use crate::models::user::User;
use crate::repositories::role_repository::RoleRepository;
use crate::models::organization::Organization;
use crate::utilities::http_request::OrganizationRequestExt;

pub async fn index(req: HttpRequest, user: User, organization: Organization, app: Data<AppState>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "roles.view");

    if req.get_query("include-permission").is_some() {
        return match RoleRepository::get_organization_roles_with_permissions(&app.pool, &organization.id).await {
            Ok(roles) => JsonResponse::success(roles),
            Err(e) => JsonResponse::fatal(e, "role_controller.index failed"),
        }
    }

    match RoleRepository::get_organization_roles(&app.pool, &organization.id).await {
        Ok(roles) => JsonResponse::success(roles),
        Err(e) => JsonResponse::fatal(e, "role_controller.index failed"),
    }
}