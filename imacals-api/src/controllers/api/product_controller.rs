use actix_multipart::Multipart;
use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use futures_util::StreamExt as _;
use serde_json::json;
use sqlx::Error;
use uuid::Uuid;

use crate::AppState;
use crate::models::organization::Organization;
use crate::models::product::{CreateProductSchema, UpdateProductSchema};
use crate::models::user::User;
use crate::repositories::product_repository::ProductRepository;
use crate::services::product_service::ProductService;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

pub async fn index(
    user: User,
    organization: Organization,
    app: Data<AppState>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "products.view");
    match ProductRepository::list_for_organization(&app.pool, &organization.id).await {
        Ok(products) => JsonResponse::success(products),
        Err(e)       => JsonResponse::fatal(e, "product_controller.index failed"),
    }
}

pub async fn show(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "products.view");
    match ProductRepository::find_admin_product_by_id(&app.pool, &id.into_inner()).await {
        Ok(p)                   => JsonResponse::success(p),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Product".into())),
        Err(e)                  => JsonResponse::fatal(e, "product_controller.show failed"),
    }
}

pub async fn create(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    body: Json<CreateProductSchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "products.create");
    match ProductService::create(&app.pool, &organization.id, &user.id, &body).await {
        Ok(product) => JsonResponse::success(product),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

pub async fn update(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<UpdateProductSchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "products.update");
    match ProductService::update(&app.pool, &id.into_inner(), &body).await {
        Ok(product)  => JsonResponse::success(product),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

pub async fn delete(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "products.delete");
    match ProductRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0)  => JsonResponse::error(ErrorBag::NotFound("Product".into())),
        Ok(_)  => JsonResponse::success(json!({ "message": "Product deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "product_controller.delete failed"),
    }
}

pub async fn upload_image(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    mut payload: Multipart,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "products.update");
    let product_id = id.into_inner();

    let mut file_bytes: Vec<u8> = Vec::new();
    let mut file_name = String::new();
    let mut mime_type = String::from("image/jpeg");

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(e) => return HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
        };

        let disposition = field.content_disposition().cloned();
        let field_name = disposition.as_ref().and_then(|d| d.get_name()).unwrap_or("");

        if field_name == "file" || field_name == "image" {
            file_name = disposition
                .as_ref()
                .and_then(|d| d.get_filename())
                .unwrap_or("product.jpg")
                .to_string();

            if let Some(ct) = field.content_type() {
                mime_type = ct.to_string();
            }

            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(b)  => file_bytes.extend_from_slice(&b),
                    Err(e) => return HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
                }
            }
        }
    }

    if file_bytes.is_empty() {
        return JsonResponse::error(ErrorBag::Validation {
            field: "file".into(),
            message: "No image file uploaded".into(),
        });
    }

    match ProductService::upload_image(
        &app.pool,
        &product_id,
        &user.id,
        &file_name,
        &file_bytes,
        &mime_type,
    )
    .await
    {
        Ok(product)  => JsonResponse::success(product),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}
