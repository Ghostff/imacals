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
