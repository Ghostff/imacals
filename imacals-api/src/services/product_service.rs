use sqlx::PgPool;
use uuid::Uuid;

use crate::models::file::{CreateFileInput, FileType};
use crate::models::product::{AdminProduct, CreateProductSchema, Product, UpdateProductSchema};
use crate::repositories::category_repository::CategoryRepository;
use crate::repositories::domain_repository::DomainRepository;
use crate::repositories::file_repository::FileRepository;
use crate::repositories::product_repository::ProductRepository;
use crate::services::storage_service::StorageService;
use crate::utilities::error_bag::ErrorBag;

pub struct ProductService;

impl ProductService {
    // Creates a new product for an organization.
    pub async fn create(
        pool: &PgPool,
        organization_id: &Uuid,
        user_id: &Uuid,
        schema: &CreateProductSchema,
    ) -> Result<AdminProduct, ErrorBag> {
        // Ensure category exists
        if CategoryRepository::find_by_id(pool, &schema.category_id).await.is_err() {
            return Err(ErrorBag::Validation {
                field: "category_id".into(),
                message: "Category does not exist".into(),
            });
        }

        // Resolve domain: use provided domain_id or default domain
        let domain_id = match schema.domain_id {
            Some(did) => did,
            None => {
                let domains = DomainRepository::index(pool)
                    .await
                    .map_err(|e| ErrorBag::InternalServerError(format!("Failed to query domains: {:?}", e)))?;
                domains.first().map(|d| d.id).ok_or_else(|| {
                    ErrorBag::InternalServerError("No domain configured in database".into())
                })?
            }
        };

        let min_order_qty = schema.min_order_quantity.unwrap_or(1).max(1);
        let in_stock = schema.in_stock.unwrap_or(true);

        let product = ProductRepository::create(
            pool,
            organization_id,
            &domain_id,
            &schema.category_id,
            user_id,
            &schema.name,
            &schema.slug,
            schema.description.as_deref(),
            &schema.unit,
            schema.unit_price_kobo,
            min_order_qty,
            in_stock,
        )
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.code().as_deref() == Some("23505") {
                    return ErrorBag::Validation {
                        field: "slug".into(),
                        message: "A product with this slug already exists".into(),
                    };
                }
            }
            ErrorBag::InternalServerError(format!("ProductRepository::create failed: {:?}", e))
        })?;

        ProductRepository::find_admin_product_by_id(pool, &product.id)
            .await
            .map_err(|e| ErrorBag::InternalServerError(format!("find_admin_product_by_id failed: {:?}", e)))
    }

    // Updates an existing product.
    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        schema: &UpdateProductSchema,
    ) -> Result<AdminProduct, ErrorBag> {
        let mut product: Product = ProductRepository::find_by_id(pool, id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => ErrorBag::NotFound("Product".into()),
                _ => ErrorBag::InternalServerError(format!("find_by_id failed: {:?}", e)),
            })?;

        if let Some(cat_id) = schema.category_id {
            if CategoryRepository::find_by_id(pool, &cat_id).await.is_err() {
                return Err(ErrorBag::Validation {
                    field: "category_id".into(),
                    message: "Category does not exist".into(),
                });
            }
            product.category_id = cat_id;
        }

        if let Some(did) = schema.domain_id {
            product.domain_id = did;
        }

        if let Some(ref name) = schema.name {
            product.name = name.clone();
        }

        if let Some(ref slug) = schema.slug {
            product.slug = slug.clone();
        }

        if schema.description.is_some() {
            product.description = schema.description.clone();
        }

        if let Some(ref unit) = schema.unit {
            product.unit = unit.clone();
        }

        if let Some(price) = schema.unit_price_kobo {
            if price <= 0 {
                return Err(ErrorBag::Validation {
                    field: "unit_price_kobo".into(),
                    message: "Price must be greater than zero kobo".into(),
                });
            }
            product.unit_price_kobo = price;
        }

        if let Some(moq) = schema.min_order_quantity {
            product.min_order_quantity = moq.max(1);
        }

        if let Some(in_stock) = schema.in_stock {
            product.in_stock = in_stock;
        }

        ProductRepository::update(pool, &product)
            .await
            .map_err(|e| {
                if let sqlx::Error::Database(ref db_err) = e {
                    if db_err.code().as_deref() == Some("23505") {
                        return ErrorBag::Validation {
                            field: "slug".into(),
                            message: "A product with this slug already exists".into(),
                        };
                    }
                }
                ErrorBag::InternalServerError(format!("ProductRepository::update failed: {:?}", e))
            })?;

        ProductRepository::find_admin_product_by_id(pool, id)
            .await
            .map_err(|e| ErrorBag::InternalServerError(format!("find_admin_product_by_id failed: {:?}", e)))
    }

    // Uploads and associates a product image via S3 / MinIO and the polymorphic files table.
    pub async fn upload_image(
        pool: &PgPool,
        product_id: &Uuid,
        user_id: &Uuid,
        file_name: &str,
        file_bytes: &[u8],
        mime_type: &str,
    ) -> Result<AdminProduct, ErrorBag> {
        // Ensure product exists
        let _ = ProductRepository::find_by_id(pool, product_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => ErrorBag::NotFound("Product".into()),
                _ => ErrorBag::InternalServerError(format!("find_by_id failed: {:?}", e)),
            })?;

        let ext = file_name.rsplit('.').next().unwrap_or("jpg");
        let relative_path = format!("uploads/products/{}/{}.{}", product_id, Uuid::new_v4(), ext);

        StorageService::upload(&relative_path, file_bytes, mime_type)
            .await
            .map_err(|e| ErrorBag::InternalServerError(format!("Storage upload failed: {}", e)))?;

        let absolute_path = StorageService::public_url(&relative_path);

        // Remove previous product-image files so only the newest remains active
        let _ = FileRepository::delete_all_for_owner_by_type(
            pool,
            "products",
            product_id,
            FileType::ProductImage.as_str(),
        )
        .await;

        let input = CreateFileInput {
            created_by: *user_id,
            fileable_type: "products".to_string(),
            fileable_id: *product_id,
            file_type: FileType::ProductImage,
            name: file_name.to_string(),
            absolute_path,
            relative_path,
            size: file_bytes.len() as i64,
            mime_type: mime_type.to_string(),
        };

        FileRepository::create(pool, &input)
            .await
            .map_err(|e| ErrorBag::InternalServerError(format!("FileRepository::create failed: {:?}", e)))?;

        ProductRepository::find_admin_product_by_id(pool, product_id)
            .await
            .map_err(|e| ErrorBag::InternalServerError(format!("find_admin_product_by_id failed: {:?}", e)))
    }
}
