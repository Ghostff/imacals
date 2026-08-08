use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use serde_json::json;
use sqlx::Error;
use uuid::Uuid;

use crate::AppState;
use crate::models::user::{CreateUserSchema, UpdateUserSchema, User};
use crate::repositories::permission_repository::PermissionRepository;
use crate::repositories::user_repository::UserRepository;
use crate::services::user_service::UserService;

pub async fn index(user: User, app: Data<AppState>) -> HttpResponse {
    crate::gate!(&app.pool, &user, "users.view");

    match UserRepository::index(&app.pool).await {
        Ok(users) => JsonResponse::success(users),
        Err(e) => JsonResponse::fatal(e, "user_controller.index failed"),
    }
}

pub async fn show(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    crate::gate!(&app.pool, &user, "users.view");

    match UserRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(target)              => JsonResponse::success(json!({ "user": target })),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("User".into())),
        Err(e)                  => JsonResponse::fatal(e, "user_controller.show failed"),
    }
}

pub async fn create(user: User, app: Data<AppState>, body: Json<CreateUserSchema>) -> HttpResponse {
    crate::gate!(&app.pool, &user, "users.create");

    let password = match body.password.as_deref() {
        Some(p) if !p.trim().is_empty() => p,
        _ => return JsonResponse::error(ErrorBag::Validation {
            field: "password".into(),
            message: "password is required".into(),
        }),
    };

    let new_user = match UserService::create(
        &app.pool, &body.first_name, &body.last_name, &body.email, password, Some(&body.role_id),
    ).await {
        Ok(u) => u,
        Err(e) => match e.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => return JsonResponse::fatal(e, ""),
            _ => return JsonResponse::error(e),
        },
    };

    // Extra grants beyond the role are a permission change, so they need the stronger gate.
    if let Some(permission_ids) = &body.permission_ids {
        if !crate::can!(&app.pool, &user, "users.manage-permissions") {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
        if let Err(e) = grant_extra(&app.pool, &new_user.id, permission_ids).await {
            return JsonResponse::fatal(e, "user_controller.create.grant_extra failed");
        }
    }

    JsonResponse::success(json!({ "user": new_user }))
}

pub async fn update(
    user: User,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<UpdateUserSchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, "users.update");

    let mut target = match UserRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(u) => u,
        Err(Error::RowNotFound) => return JsonResponse::error(ErrorBag::NotFound("User".into())),
        Err(e) => return JsonResponse::fatal(e, "user_controller.update.find_by_id failed"),
    };

    let email = body.email.trim().to_lowercase();
    if target.email != email && UserRepository::email_exist(&app.pool, &email).await.unwrap_or(false) {
        return JsonResponse::error(ErrorBag::EmailInUse);
    }

    target.first_name    = body.first_name.clone();
    target.last_name     = body.last_name.clone();
    target.email         = email;
    target.phone         = body.phone.clone();
    target.date_of_birth = body.date_of_birth;

    // Changing someone's role rewrites what they can do, so it needs the permission gate rather
    // than plain users.update.
    let role_changed = body.role_id.is_some() && body.role_id != target.role_id;
    if role_changed || body.permission_ids.is_some() {
        if !crate::can!(&app.pool, &user, "users.manage-permissions") {
            return JsonResponse::error(ErrorBag::Forbidden);
        }
    }
    if let Some(role_id) = body.role_id {
        target.role_id = Some(role_id);
    }

    if let Err(e) = UserRepository::update(&app.pool, &target).await {
        return JsonResponse::fatal(e, "user_controller.update.update failed");
    }

    // Re-sync from the new role first, so the role's bundle replaces the old one, then layer any
    // explicit extras on top.
    if role_changed {
        if let Some(role_id) = target.role_id {
            if let Err(e) = PermissionRepository::sync_from_role(&app.pool, &target.id, &role_id).await {
                return JsonResponse::fatal(e, "user_controller.update.sync_from_role failed");
            }
        }
    }

    if let Some(permission_ids) = &body.permission_ids {
        if let Err(e) = grant_extra(&app.pool, &target.id, permission_ids).await {
            return JsonResponse::fatal(e, "user_controller.update.grant_extra failed");
        }
    }

    JsonResponse::success(json!({ "message": "User updated successfully" }))
}

pub async fn delete(user: User, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    crate::gate!(&app.pool, &user, "users.delete");

    match UserRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0)  => JsonResponse::error(ErrorBag::NotFound("User".into())),
        Ok(_)  => JsonResponse::success(json!({ "message": "User deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "user_controller.delete failed"),
    }
}

// Adds grants on top of whatever the role already gave. Revives a soft-deleted row rather than
// inserting a duplicate, which the partial unique index would reject.
async fn grant_extra(pool: &sqlx::PgPool, user_id: &Uuid, permission_ids: &[Uuid]) -> Result<(), Error> {
    for permission_id in permission_ids {
        sqlx::query!(
            r#"
            INSERT INTO user_permissions (user_id, permission_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, permission_id) WHERE deleted_at IS NULL DO NOTHING
            "#,
            user_id,
            permission_id
        ).execute(pool).await?;

        sqlx::query!(
            "UPDATE user_permissions SET deleted_at = NULL, updated_at = NOW()
             WHERE user_id = $1 AND permission_id = $2 AND deleted_at IS NOT NULL
               AND NOT EXISTS (
                   SELECT 1 FROM user_permissions x
                   WHERE x.user_id = $1 AND x.permission_id = $2 AND x.deleted_at IS NULL
               )",
            user_id,
            permission_id
        ).execute(pool).await?;
    }
    Ok(())
}
