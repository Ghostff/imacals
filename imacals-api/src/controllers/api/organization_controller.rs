use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use validator::Validate;

use crate::AppState;
use crate::models::organization::Organization;
use crate::models::user::User;
use crate::repositories::organization_repository::OrganizationRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrganizationSchema {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
    pub parent_id: Option<Uuid>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOrganizationSchema {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 100))]
    pub slug: String,
    pub parent_id: Option<Uuid>,
    pub description: Option<String>,
}

pub async fn index(user: User, organization: Organization, app: Data<AppState>) -> HttpResponse {
    match OrganizationRepository::get_organizations(&app.pool, &user, &organization).await {
        Ok(o) => JsonResponse::success(o),
        Err(e) => JsonResponse::fatal(e, "organization_controller.index failed"),
    }
}

pub async fn show(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    if !user.is_internal && !user.is_superuser {
        return JsonResponse::error(ErrorBag::Forbidden);
    }

    match OrganizationRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(o) => JsonResponse::success(o),
        Err(sqlx::Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Organization".into())),
        Err(e) => JsonResponse::fatal(e, "organization_controller.show failed"),
    }
}

pub async fn create(
    user: User,
    app: Data<AppState>,
    body: Json<CreateOrganizationSchema>,
) -> HttpResponse {
    if !user.is_internal && !user.is_superuser {
        return JsonResponse::error(ErrorBag::Forbidden);
    }

    match OrganizationRepository::create(
        &app.pool,
        &body.name,
        &body.slug,
        body.parent_id,
        body.description.as_deref(),
        &user.id,
    )
    .await
    {
        Ok(o) => JsonResponse::success(o),
        Err(e) => JsonResponse::fatal(e, "organization_controller.create failed"),
    }
}

pub async fn update(
    user: User,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<UpdateOrganizationSchema>,
) -> HttpResponse {
    if !user.is_internal && !user.is_superuser {
        return JsonResponse::error(ErrorBag::Forbidden);
    }

    let mut org = match OrganizationRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(o) => o,
        Err(sqlx::Error::RowNotFound) => {
            return JsonResponse::error(ErrorBag::NotFound("Organization".into()))
        }
        Err(e) => return JsonResponse::fatal(e, "organization_controller.update.find_by_id failed"),
    };

    org.name = body.name.clone();
    org.slug = body.slug.clone();
    org.parent_id = body.parent_id;
    org.description = body.description.clone();

    match OrganizationRepository::update(&app.pool, &org).await {
        Ok(_) => JsonResponse::success(json!({ "message": "Organization updated successfully" })),
        Err(e) => JsonResponse::fatal(e, "organization_controller.update failed"),
    }
}

pub async fn delete(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    if !user.is_internal && !user.is_superuser {
        return JsonResponse::error(ErrorBag::Forbidden);
    }

    match OrganizationRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0) => JsonResponse::error(ErrorBag::NotFound("Organization".into())),
        Ok(_) => JsonResponse::success(json!({ "message": "Organization deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "organization_controller.delete failed"),
    }
}
