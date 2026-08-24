use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::models::category::Category;

pub struct CategoryRepository;

impl CategoryRepository {
    // List all active categories, optionally scoped to a domain.
    pub async fn list_all(pool: &PgPool, domain_id: Option<&Uuid>) -> Result<Vec<Category>, Error> {
        match domain_id {
            Some(did) => {
                sqlx::query_as!(
                    Category,
                    r#"SELECT id, domain_id, created_by, name, slug, description,
                              created_at, updated_at, deleted_at
                       FROM categories
                       WHERE domain_id = $1 AND deleted_at IS NULL
                       ORDER BY name ASC"#,
                    did
                )
                .fetch_all(pool)
                .await
            }
            None => {
                sqlx::query_as!(
                    Category,
                    r#"SELECT id, domain_id, created_by, name, slug, description,
                              created_at, updated_at, deleted_at
                       FROM categories
                       WHERE deleted_at IS NULL
                       ORDER BY name ASC"#
                )
                .fetch_all(pool)
                .await
            }
        }
    }

    // Returns an error if the category doesn't exist or is soft-deleted.
    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Category, Error> {
        sqlx::query_as!(
            Category,
            r#"SELECT id, domain_id, created_by, name, slug, description,
                      created_at, updated_at, deleted_at
               FROM categories
               WHERE id = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            id
        )
        .fetch_one(pool)
        .await
    }

    // Find category by slug across active categories.
    pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Category, Error> {
        sqlx::query_as!(
            Category,
            r#"SELECT id, domain_id, created_by, name, slug, description,
                      created_at, updated_at, deleted_at
               FROM categories
               WHERE slug = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            slug
        )
        .fetch_one(pool)
        .await
    }

    // Create a new category row.
    pub async fn create(
        pool: &PgPool,
        domain_id: &Uuid,
        created_by: Option<&Uuid>,
        name: &str,
        slug: &str,
        description: Option<&str>,
    ) -> Result<Category, Error> {
        sqlx::query_as!(
            Category,
            r#"INSERT INTO categories (domain_id, created_by, name, slug, description)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, domain_id, created_by, name, slug, description,
                         created_at, updated_at, deleted_at"#,
            domain_id,
            created_by,
            name,
            slug,
            description
        )
        .fetch_one(pool)
        .await
    }

    // Update an existing category.
    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        name: &str,
        slug: &str,
        description: Option<&str>,
    ) -> Result<Category, Error> {
        sqlx::query_as!(
            Category,
            r#"UPDATE categories
               SET name = $2, slug = $3, description = $4, updated_at = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, domain_id, created_by, name, slug, description,
                         created_at, updated_at, deleted_at"#,
            id,
            name,
            slug,
            description
        )
        .fetch_one(pool)
        .await
    }

    // Soft-delete category.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE categories SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}
