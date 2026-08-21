use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde_json::json;
use sqlx::Error;
use uuid::Uuid;

use crate::AppState;
use crate::models::category::{CreateCategorySchema, UpdateCategorySchema};
use crate::models::organization::Organization;
use crate::models::user::User;
use crate::repositories::category_repository::CategoryRepository;
use crate::repositories::domain_repository::DomainRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

pub async fn index(
    user: User,
    organization: Organization,
    app: Data<AppState>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "categories.view");
    match CategoryRepository::list_all(&app.pool, None).await {
        Ok(categories) => JsonResponse::success(categories),
        Err(e)         => JsonResponse::fatal(e, "category_controller.index failed"),
    }
}

pub async fn show(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "categories.view");
    match CategoryRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(c)                   => JsonResponse::success(c),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Category".into())),
        Err(e)                  => JsonResponse::fatal(e, "category_controller.show failed"),
    }
}

pub async fn create(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    body: Json<CreateCategorySchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "categories.create");

    let domain_id = match body.domain_id {
        Some(did) => did,
        None => {
            let domains = match DomainRepository::index(&app.pool).await {
                Ok(d) => d,
                Err(e) => return JsonResponse::fatal(e, "category_controller.create failed to get domains"),
            };
            match domains.first() {
                Some(d) => d.id,
                None => return JsonResponse::error(ErrorBag::InternalServerError("No domain configured".into())),
            }
        }
    };

    match CategoryRepository::create(
        &app.pool,
        &domain_id,
        Some(&user.id),
        &body.name,
        &body.slug,
        body.description.as_deref(),
    )
    .await
    {
        Ok(c) => JsonResponse::success(c),
        Err(Error::Database(ref db_err)) if db_err.code().as_deref() == Some("23505") => {
            JsonResponse::error(ErrorBag::Validation {
                field: "slug".into(),
                message: "A category with this slug already exists".into(),
            })
        }
        Err(e) => JsonResponse::fatal(e, "category_controller.create failed"),
    }
}

pub async fn update(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<UpdateCategorySchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "categories.update");
    let category_id = id.into_inner();

    match CategoryRepository::update(
        &app.pool,
        &category_id,
        &body.name,
        &body.slug,
        body.description.as_deref(),
    )
    .await
    {
        Ok(c)                   => JsonResponse::success(c),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Category".into())),
        Err(Error::Database(ref db_err)) if db_err.code().as_deref() == Some("23505") => {
            JsonResponse::error(ErrorBag::Validation {
                field: "slug".into(),
                message: "A category with this slug already exists".into(),
            })
        }
        Err(e)                  => JsonResponse::fatal(e, "category_controller.update failed"),
    }
}

pub async fn delete(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "categories.delete");
    match CategoryRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0)  => JsonResponse::error(ErrorBag::NotFound("Category".into())),
        Ok(_)  => JsonResponse::success(json!({ "message": "Category deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "category_controller.delete failed"),
    }
}
