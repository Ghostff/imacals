use actix_web::web::{Data, Path, Query};
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use sqlx::Error;

use crate::AppState;
use crate::repositories::category_repository::CategoryRepository;
use crate::repositories::product_repository::ProductRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

#[derive(Debug, Deserialize)]
pub struct CatalogQuery {
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicCategory {
    pub slug: String,
    pub name: String,
}

// Public endpoint: list products in the active catalogue, optionally filtered by category slug.
pub async fn products(
    app: Data<AppState>,
    query: Query<CatalogQuery>,
) -> HttpResponse {
    let cat_filter = query.category.as_deref().filter(|s| !s.is_empty());
    match ProductRepository::list_for_catalog(&app.pool, cat_filter).await {
        Ok(items) => JsonResponse::success(items),
        Err(e)    => JsonResponse::fatal(e, "catalog_controller.products failed"),
    }
}

// Public endpoint: retrieve single product by slug for product details view.
pub async fn show_by_slug(
    app: Data<AppState>,
    slug: Path<String>,
) -> HttpResponse {
    match ProductRepository::find_by_slug_for_catalog(&app.pool, &slug.into_inner()).await {
        Ok(p)                   => JsonResponse::success(p),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Product".into())),
        Err(e)                  => JsonResponse::fatal(e, "catalog_controller.show_by_slug failed"),
    }
}

// Public endpoint: list all active product categories for navigation filters.
pub async fn categories(
    app: Data<AppState>,
) -> HttpResponse {
    match CategoryRepository::list_all(&app.pool, None).await {
        Ok(cats) => {
            let public_cats: Vec<PublicCategory> = cats
                .into_iter()
                .map(|c| PublicCategory { slug: c.slug, name: c.name })
                .collect();
            JsonResponse::success(public_cats)
        }
        Err(e) => JsonResponse::fatal(e, "catalog_controller.categories failed"),
    }
}
