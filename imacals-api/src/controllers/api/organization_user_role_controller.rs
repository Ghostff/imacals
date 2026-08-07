use actix_web::web::Data;
use actix_web::HttpResponse;
use crate::AppState;
use crate::models::user::User;
use crate::models::organization::Organization;
use crate::repositories::organization_user_role_repository::OrganizationUserRoleRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

pub async fn index(user: User, organization: Organization, app: Data<AppState>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.view");
    match OrganizationUserRoleRepository::index(&app.pool, &organization.id).await {
        Ok(roles) => JsonResponse::success(roles),
        Err(e) => JsonResponse::fatal(e, "organization_user_role_controller.index failed"),
    }
}
