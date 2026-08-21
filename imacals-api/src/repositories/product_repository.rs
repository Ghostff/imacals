use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::models::product::{AdminProduct, CatalogProduct, Product};

pub struct ProductRepository;

impl ProductRepository {
    // List all products for the customer storefront catalogue with joined category and image.
    pub async fn list_for_catalog(
        pool: &PgPool,
        category_slug: Option<&str>,
    ) -> Result<Vec<CatalogProduct>, Error> {
        match category_slug {
            Some(cat_slug) => {
                sqlx::query_as!(
                    CatalogProduct,
                    r#"SELECT
                        p.id::text as "id!",
                        p.slug,
                        p.name,
                        COALESCE(p.description, '') as "description!",
                        c.slug as "category_slug!",
                        c.name as "category_name!",
                        p.unit,
                        p.unit_price_kobo,
                        p.min_order_quantity,
                        p.in_stock,
                        f.absolute_path as "image_url?"
                    FROM products p
                    JOIN categories c ON c.id = p.category_id AND c.deleted_at IS NULL
                    LEFT JOIN LATERAL (
                        SELECT absolute_path FROM files
                        WHERE fileable_type = 'products'
                          AND fileable_id = p.id
                          AND deleted_at IS NULL
                        ORDER BY created_at DESC
                        LIMIT 1
                    ) f ON true
                    WHERE p.deleted_at IS NULL
                      AND c.slug = $1
                    ORDER BY p.created_at DESC"#,
                    cat_slug
                )
                .fetch_all(pool)
                .await
            }
            None => {
                sqlx::query_as!(
                    CatalogProduct,
                    r#"SELECT
                        p.id::text as "id!",
                        p.slug,
                        p.name,
                        COALESCE(p.description, '') as "description!",
                        c.slug as "category_slug!",
                        c.name as "category_name!",
                        p.unit,
                        p.unit_price_kobo,
                        p.min_order_quantity,
                        p.in_stock,
                        f.absolute_path as "image_url?"
                    FROM products p
                    JOIN categories c ON c.id = p.category_id AND c.deleted_at IS NULL
                    LEFT JOIN LATERAL (
                        SELECT absolute_path FROM files
                        WHERE fileable_type = 'products'
                          AND fileable_id = p.id
                          AND deleted_at IS NULL
                        ORDER BY created_at DESC
                        LIMIT 1
                    ) f ON true
                    WHERE p.deleted_at IS NULL
                    ORDER BY p.created_at DESC"#
                )
                .fetch_all(pool)
                .await
            }
        }
    }

    // Find a single product by slug for storefront display.
    pub async fn find_by_slug_for_catalog(pool: &PgPool, slug: &str) -> Result<CatalogProduct, Error> {
        sqlx::query_as!(
            CatalogProduct,
            r#"SELECT
                p.id::text as "id!",
                p.slug,
                p.name,
                COALESCE(p.description, '') as "description!",
                c.slug as "category_slug!",
                c.name as "category_name!",
                p.unit,
                p.unit_price_kobo,
                p.min_order_quantity,
                p.in_stock,
                f.absolute_path as "image_url?"
            FROM products p
            JOIN categories c ON c.id = p.category_id AND c.deleted_at IS NULL
            LEFT JOIN LATERAL (
                SELECT absolute_path FROM files
                WHERE fileable_type = 'products'
                  AND fileable_id = p.id
                  AND deleted_at IS NULL
                ORDER BY created_at DESC
                LIMIT 1
            ) f ON true
            WHERE p.slug = $1
              AND p.deleted_at IS NULL
            LIMIT 1"#,
            slug
        )
        .fetch_one(pool)
        .await
    }

    // List products for the back-office dashboard scoped to an organization.
    pub async fn list_for_organization(
        pool: &PgPool,
        organization_id: &Uuid,
    ) -> Result<Vec<AdminProduct>, Error> {
        sqlx::query_as!(
            AdminProduct,
            r#"SELECT
                p.id,
                p.organization_id,
                p.domain_id,
                p.category_id,
                c.name as "category_name!",
                c.slug as "category_slug!",
                p.created_by,
                p.name,
                p.slug,
                p.description,
                p.unit,
                p.unit_price_kobo,
                p.min_order_quantity,
                p.in_stock,
                f.absolute_path as "image_url?",
                p.created_at,
                p.updated_at
            FROM products p
            JOIN categories c ON c.id = p.category_id AND c.deleted_at IS NULL
            LEFT JOIN LATERAL (
                SELECT absolute_path FROM files
                WHERE fileable_type = 'products'
                  AND fileable_id = p.id
                  AND deleted_at IS NULL
                ORDER BY created_at DESC
                LIMIT 1
            ) f ON true
            WHERE p.organization_id = $1
              AND p.deleted_at IS NULL
            ORDER BY p.created_at DESC"#,
            organization_id
        )
        .fetch_all(pool)
        .await
    }

    // Find raw product record by id.
    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Product, Error> {
        sqlx::query_as!(
            Product,
            r#"SELECT id, organization_id, domain_id, category_id, created_by,
                      name, slug, description, unit, unit_price_kobo,
                      min_order_quantity, in_stock, created_at, updated_at, deleted_at
               FROM products
               WHERE id = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            id
        )
        .fetch_one(pool)
        .await
    }

    // Find detailed admin product by id.
    pub async fn find_admin_product_by_id(pool: &PgPool, id: &Uuid) -> Result<AdminProduct, Error> {
        sqlx::query_as!(
            AdminProduct,
            r#"SELECT
                p.id,
                p.organization_id,
                p.domain_id,
                p.category_id,
                c.name as "category_name!",
                c.slug as "category_slug!",
                p.created_by,
                p.name,
                p.slug,
                p.description,
                p.unit,
                p.unit_price_kobo,
                p.min_order_quantity,
                p.in_stock,
                f.absolute_path as "image_url?",
                p.created_at,
                p.updated_at
            FROM products p
            JOIN categories c ON c.id = p.category_id AND c.deleted_at IS NULL
            LEFT JOIN LATERAL (
                SELECT absolute_path FROM files
                WHERE fileable_type = 'products'
                  AND fileable_id = p.id
                  AND deleted_at IS NULL
                ORDER BY created_at DESC
                LIMIT 1
            ) f ON true
            WHERE p.id = $1
              AND p.deleted_at IS NULL
            LIMIT 1"#,
            id
        )
        .fetch_one(pool)
        .await
    }

    // Create a new product.
    pub async fn create(
        pool: &PgPool,
        organization_id: &Uuid,
        domain_id: &Uuid,
        category_id: &Uuid,
        created_by: &Uuid,
        name: &str,
        slug: &str,
        description: Option<&str>,
        unit: &str,
        unit_price_kobo: i64,
        min_order_quantity: i32,
        in_stock: bool,
    ) -> Result<Product, Error> {
        sqlx::query_as!(
            Product,
            r#"INSERT INTO products
                   (organization_id, domain_id, category_id, created_by,
                    name, slug, description, unit, unit_price_kobo,
                    min_order_quantity, in_stock)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING id, organization_id, domain_id, category_id, created_by,
                         name, slug, description, unit, unit_price_kobo,
                         min_order_quantity, in_stock, created_at, updated_at, deleted_at"#,
            organization_id,
            domain_id,
            category_id,
            created_by,
            name,
            slug,
            description,
            unit,
            unit_price_kobo,
            min_order_quantity,
            in_stock
        )
        .fetch_one(pool)
        .await
    }

    // Update an existing product.
    pub async fn update(pool: &PgPool, product: &Product) -> Result<Product, Error> {
        sqlx::query_as!(
            Product,
            r#"UPDATE products
               SET domain_id = $2,
                   category_id = $3,
                   name = $4,
                   slug = $5,
                   description = $6,
                   unit = $7,
                   unit_price_kobo = $8,
                   min_order_quantity = $9,
                   in_stock = $10,
                   updated_at = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, organization_id, domain_id, category_id, created_by,
                         name, slug, description, unit, unit_price_kobo,
                         min_order_quantity, in_stock, created_at, updated_at, deleted_at"#,
            product.id,
            product.domain_id,
            product.category_id,
            product.name,
            product.slug,
            product.description,
            product.unit,
            product.unit_price_kobo,
            product.min_order_quantity,
            product.in_stock
        )
        .fetch_one(pool)
        .await
    }

    // Soft-delete product.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE products SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}
