use actix_web::{HttpResponse};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use serde_json::json;
use sqlx::{Error};
use uuid::Uuid;

use crate::AppState;
use crate::models::organization::Organization;
use crate::models::user::{CreateUserSchema, UpdateUserSchema, User};
use crate::repositories::user_repository::UserRepository;
use crate::repositories::organization_repository::OrganizationRepository;
use crate::repositories::organization_user_permission_repository::OrganizationUserPermissionRepository;
use crate::repositories::organization_user_repository::OrganizationUserRepository;
use crate::services::user_service::UserService;

pub async fn index(user: User, organization: Organization, app: Data<AppState>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.view");

    match UserRepository::get_for_organization(&app.pool, &organization, &user).await {
        Ok(users) => JsonResponse::success(users),
        Err(e) => JsonResponse::fatal(e, "user_controller.index.find_all_for_organization failed"),
    }
}

pub async fn show(user: User, organization: Organization, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.view");

    let target_user_id = id.into_inner();
    let target_user = match UserRepository::find_by_id(&app.pool, &target_user_id).await {
        Ok(u) => u,
        Err(Error::RowNotFound) => return JsonResponse::error(ErrorBag::NotFound("User".into())),
        Err(e) => return JsonResponse::fatal(e, "user_controller.show.find_by_id failed"),
    };

    let organizations = match OrganizationRepository::get_all_with_permissions_for_user(&app.pool, &target_user).await {
        Ok(p) => p,
        Err(e) => return JsonResponse::fatal(e, "user_controller.show.get_all_with_permissions_for_user failed"),
    };

    JsonResponse::success(json!({ "user": target_user, "organizations": organizations }))
}

pub async fn update(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<UpdateUserSchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.update");

    let mut target_user = match UserRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(u) => u,
        Err(e) => return JsonResponse::fatal(e, "user_controller.update.find_by_id failed"),
    };

    let email = body.email.clone().trim().to_lowercase();
    if target_user.email != email && UserRepository::email_exist(&app.pool, &email).await.unwrap_or(false) {
        return JsonResponse::error(ErrorBag::EmailInUse);
    }

    target_user.first_name    = body.first_name.clone();
    target_user.last_name     = body.last_name.clone();
    target_user.email         = email;
    target_user.phone         = body.phone.clone();
    target_user.date_of_birth = body.date_of_birth;

    if let Err(e) = UserRepository::update(&app.pool, &target_user).await {
        return JsonResponse::fatal(e, "user_controller.update.update failed");
    }

    if let Some(org_ids) = &body.organization_ids {
        if let Err(e) = OrganizationUserRepository::sync_user_organizations_and_permissions(&app.pool, &target_user, &org_ids, &user).await {
            return JsonResponse::fatal(e, "user_controller.update.sync_user_organizations failed");
        }
    }

    // @todo: improve
    if let Some(permission_ids) = &body.permission_ids {
        if !crate::can!(&app.pool, &user, &organization, "roles.update") {
            return JsonResponse::error(ErrorBag::Forbidden);
        }

        let ou = match sqlx::query!("SELECT id FROM organization_users WHERE user_id = $1 AND organization_id = $2 AND deleted_at IS NULL", target_user.id, organization.id).fetch_one(&app.pool).await {
            Ok(o) => o,
            Err(e) => return JsonResponse::fatal(e, "user_controller.update.find_organization_user failed"),
        };

        if let Err(e) = OrganizationUserPermissionRepository::sync_permissions(&app.pool, &ou.id, permission_ids).await {
            return JsonResponse::fatal(e, "user_controller.update.sync_permissions failed");
        }
    }

    JsonResponse::success(json!({ "message": "User updated successfully" }))
}

pub async fn create(user: User, organization: Organization, app: Data<AppState>, mut body: Json<CreateUserSchema>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.create");

    let mut organization_ids = body.organization_ids.clone().unwrap_or_default();
    // When no organization is specified, create the user in the organization of the logged in user
    if organization_ids.is_empty() {
        organization_ids.push(organization.id);
    }

    // Only user.is_internal and organization == 'imacals' can add users to different orgs.
    if organization_ids.len() > 1 || (organization_ids.len() == 1 && organization_ids[0] != organization.id) {
        if !user.is_internal || organization.slug != "imacals" {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    }

    if organization_ids.contains(&organization.id) {
        body.is_internal = organization.slug == "imacals";
    }

    let new_user = match UserService::create(&app.pool, &body).await {
        Ok(u) => u,
        Err(e) => match e.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => return JsonResponse::fatal(e, ""),
            _ => return JsonResponse::error(e),
        },
    };

    // Check users.create permission for orgs other than the current context
    for org_id in &organization_ids {
        if *org_id != organization.id {
            let target_org = match OrganizationRepository::find_by_id(&app.pool, org_id).await {
                Ok(o) => o,
                Err(e) => return JsonResponse::fatal(e, "user_controller.create.find_by_id failed"),
            };
            if !crate::can!(&app.pool, &user, &target_org, "users.create") {
                return JsonResponse::error(ErrorBag::Forbidden);
            }
        }
    }

    // Add new user to organizations
    if let Err(e) = OrganizationUserRepository::sync_user_organizations_and_permissions(
        &app.pool, &new_user, &organization_ids, &user,
    ).await {
        return JsonResponse::fatal(e, "user_controller.create.sync_organizations failed");
    }

    for org_id in &organization_ids {
        if let Err(e) = sqlx::query!(
            "UPDATE organization_users SET user_role_id = $1 WHERE user_id = $2 AND organization_id = $3 AND deleted_at IS NULL",
            body.user_role_id,
            new_user.id,
            org_id
        ).execute(&app.pool).await {
            return JsonResponse::fatal(e, "user_controller.create.set_user_role failed");
        }
    }

    // Assign permissions if provided (requires roles.update in the current org)
    if let Some(permission_ids) = &body.permission_ids {
        if !crate::can!(&app.pool, &user, &organization, "roles.update") {
            return JsonResponse::error(ErrorBag::Forbidden);
        }

        for org_id in &organization_ids {
            let ou = match sqlx::query!(
                "SELECT id FROM organization_users WHERE user_id = $1 AND organization_id = $2 AND deleted_at IS NULL",
                new_user.id,
                org_id
            ).fetch_one(&app.pool).await {
                Ok(o) => o,
                Err(e) => return JsonResponse::fatal(e, "user_controller.create.find_organization_user failed"),
            };

            if let Err(e) = OrganizationUserPermissionRepository::sync_permissions(&app.pool, &ou.id, permission_ids).await {
                return JsonResponse::fatal(e, "user_controller.create.sync_permissions failed");
            }
        }
    }

    JsonResponse::success(json!({ "user": new_user }))
}

pub async fn delete(user: User, organization: Organization, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.delete");

    match UserRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0) => JsonResponse::error(ErrorBag::NotFound("User".into())),
        Ok(_) => JsonResponse::success(json!({ "message": "User deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "user_controller.delete failed"),
    }
}
