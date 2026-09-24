use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::models::file::{CreateFileInput, File, FileType};

pub struct FileRepository;

impl FileRepository {
    pub async fn find_for_owner(
        pool: &PgPool,
        fileable_type: &str,
        fileable_id: &Uuid,
    ) -> Result<Vec<File>, Error> {
        sqlx::query_as!(
            File,
            r#"SELECT id, created_by, fileable_type, fileable_id,
                      "type" AS "file_type: FileType",
                      name, absolute_path, relative_path,
                      size, mime_type, created_at, updated_at, deleted_at
               FROM files
               WHERE fileable_type = $1
                 AND fileable_id   = $2
                 AND deleted_at IS NULL
               ORDER BY created_at DESC"#,
            fileable_type,
            fileable_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<File, Error> {
        sqlx::query_as!(
            File,
            r#"SELECT id, created_by, fileable_type, fileable_id,
                      "type" AS "file_type: FileType",
                      name, absolute_path, relative_path,
                      size, mime_type, created_at, updated_at, deleted_at
               FROM files
               WHERE id = $1 AND deleted_at IS NULL"#,
            id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn create(pool: &PgPool, input: &CreateFileInput) -> Result<File, Error> {
        sqlx::query_as!(
            File,
            r#"INSERT INTO files
                   (created_by, fileable_type, fileable_id, "type", name,
                    absolute_path, relative_path, size, mime_type)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING id, created_by, fileable_type, fileable_id,
                         "type" AS "file_type: FileType",
                         name, absolute_path, relative_path,
                         size, mime_type, created_at, updated_at, deleted_at"#,
            input.created_by,
            input.fileable_type,
            input.fileable_id,
            input.file_type.as_str(),
            input.name,
            input.absolute_path,
            input.relative_path,
            input.size,
            input.mime_type,
        )
        .fetch_one(pool)
        .await
    }

    // Find all images for a product with default image first.
    pub async fn find_product_images(
        pool: &PgPool,
        product_id: &Uuid,
    ) -> Result<Vec<File>, Error> {
        sqlx::query_as!(
            File,
            r#"SELECT id, created_by, fileable_type, fileable_id,
                      "type" AS "file_type: FileType",
                      name, absolute_path, relative_path,
                      size, mime_type, created_at, updated_at, deleted_at
               FROM files
               WHERE fileable_type = 'products'
                 AND fileable_id   = $1
                 AND "type" IN ('product-image', 'product-image-default')
                 AND deleted_at IS NULL
               ORDER BY CASE WHEN "type" = 'product-image-default' THEN 0 ELSE 1 END, created_at ASC"#,
            product_id,
        )
        .fetch_all(pool)
        .await
    }

    // Sets a specific product image as the default (and demotes previous default).
    pub async fn set_default_product_image(
        pool: &PgPool,
        product_id: &Uuid,
        file_id: &Uuid,
    ) -> Result<(), Error> {
        let mut tx = pool.begin().await?;

        sqlx::query!(
            "UPDATE files SET type = 'product-image'
             WHERE fileable_type = 'products'
               AND fileable_id   = $1
               AND type          = 'product-image-default'
               AND deleted_at IS NULL",
            product_id,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE files SET type = 'product-image-default'
             WHERE id            = $1
               AND fileable_type = 'products'
               AND fileable_id   = $2
               AND deleted_at IS NULL",
            file_id,
            product_id,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    // Soft-deletes a product image and automatically promotes next available image if needed.
    pub async fn delete_product_image(
        pool: &PgPool,
        product_id: &Uuid,
        file_id: &Uuid,
    ) -> Result<u64, Error> {
        let mut tx = pool.begin().await?;

        let current = sqlx::query!(
            "SELECT type FROM files
             WHERE id = $1 AND fileable_type = 'products' AND fileable_id = $2 AND deleted_at IS NULL",
            file_id,
            product_id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let was_default = current.map(|r| r.r#type == "product-image-default").unwrap_or(false);

        let rows = sqlx::query!(
            "UPDATE files SET deleted_at = NOW()
             WHERE id = $1
               AND fileable_type = 'products'
               AND fileable_id   = $2
               AND deleted_at IS NULL",
            file_id,
            product_id,
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if was_default && rows > 0 {
            sqlx::query!(
                "UPDATE files SET type = 'product-image-default'
                 WHERE id = (
                     SELECT id FROM files
                     WHERE fileable_type = 'products'
                       AND fileable_id   = $1
                       AND deleted_at IS NULL
                     ORDER BY created_at ASC
                     LIMIT 1
                 )",
                product_id,
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(rows)
    }

    // Replaces all files of a given type for an owner — used to enforce single-image slots.
    pub async fn delete_all_for_owner_by_type(
        pool: &PgPool,
        fileable_type: &str,
        fileable_id: &Uuid,
        file_type: &str,
    ) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE files SET deleted_at = NOW()
             WHERE fileable_type = $1
               AND fileable_id   = $2
               AND type          = $3
               AND deleted_at IS NULL",
            fileable_type,
            fileable_id,
            file_type,
        )
        .execute(pool)
        .await?
        .rows_affected())
    }

    // Soft-delete scoped to owner so a user cannot delete another owner's file.
    pub async fn delete_for_owner(
        pool: &PgPool,
        id: &Uuid,
        fileable_type: &str,
        fileable_id: &Uuid,
    ) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE files SET deleted_at = NOW()
             WHERE id = $1
               AND fileable_type = $2
               AND fileable_id   = $3
               AND deleted_at IS NULL",
            id,
            fileable_type,
            fileable_id,
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}
