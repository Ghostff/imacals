use crate::models::organization::{Organization, OrganizationWithPermissions};
use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::user::User;

pub struct OrganizationRepository;

impl OrganizationRepository {

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Organization, Error> {
        Ok(
            sqlx::query_as!(
                Organization,
                "SELECT * FROM organizations WHERE id = $1 AND deleted_at IS NULL",
                id
            ).fetch_one(pool).await?
        )
    }

    pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Organization, Error> {
        Ok(
            sqlx::query_as!(
                Organization,
                "SELECT * FROM organizations WHERE slug = $1 AND deleted_at IS NULL",
                slug
            ).fetch_one(pool).await?
        )
    }

    pub async fn get(pool: &PgPool) -> Result<Vec<Organization>, Error> {
        Ok(
            sqlx::query_as!(
                Organization,
                "SELECT * FROM organizations WHERE deleted_at IS NULL"
            ).fetch_all(pool).await?
        )
    }

    pub async fn get_organizations(pool: &PgPool, user: &User, organization: &Organization) -> Result<Vec<Organization>, Error> {
        if user.is_internal && organization.is_imacals() {
            return OrganizationRepository::get(pool).await;
        }

        // If internal user is viewing organization, show the organization and its children
        if user.is_internal {
            return Ok(sqlx::query_as!(
                Organization,
                r#"
                SELECT * FROM organizations
                WHERE (id = $1 OR (parent_id IS NOT NULL AND parent_id = $1)) AND deleted_at IS NULL
                "#,
                &organization.id
            ).fetch_all(pool).await?)
        }

        // If user is not internal, show only the organization they are a member of
        Ok(
            sqlx::query_as!(
                Organization,
                r#"
                SELECT o.* FROM organizations o
                INNER JOIN organization_users ou ON o.id = ou.organization_id
                WHERE ou.user_id = $1 AND o.deleted_at IS NULL AND ou.deleted_at IS NULL
                "#,
                &user.id
            ).fetch_all(pool).await?
        )
    }

    pub async fn get_all_with_permissions_for_user(
        pool: &PgPool,
        user: &User,
    ) -> Result<Vec<OrganizationWithPermissions>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                o.id, o.name, o.parent_id, o.description, o.slug, o.created_by, o.created_at, o.updated_at, o.deleted_at,
                COALESCE(json_agg(json_build_object(
                    'id', p.id,
                    'name', p.name
                )) FILTER (WHERE p.id IS NOT NULL), '[]') AS "permissions!: serde_json::Value"
            FROM organizations o
            LEFT JOIN organization_users ou ON o.id = ou.organization_id AND ou.user_id = $1 AND ou.deleted_at IS NULL
            LEFT JOIN organization_users_permissions oup ON ou.id = oup.organization_users_id AND oup.deleted_at IS NULL
            LEFT JOIN permissions p ON oup.permission_id = p.id
            WHERE o.deleted_at IS NULL AND ($2 = TRUE OR ou.id IS NOT NULL)
            GROUP BY o.id
            ORDER BY o.created_at ASC
            "#,
            &user.id,
            user.is_internal,
        ).fetch_all(pool).await?;

        Ok(rows.into_iter().map(|row| OrganizationWithPermissions {
            organization: Organization {
                id: row.id,
                name: row.name,
                parent_id: row.parent_id,
                description: row.description,
                slug: row.slug,
                created_by: row.created_by,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            },
            permissions: serde_json::from_value(row.permissions).unwrap_or_default(),
        }).collect())
    }

    pub async fn create(pool: &PgPool, name: &str, slug: &str, parent_id: Option<Uuid>, description: Option<&str>, created_by: &Uuid) -> Result<Organization, Error> {
        Ok(
            sqlx::query_as!(
                Organization,
                r#"
                INSERT INTO organizations (name, slug, parent_id, description, created_by)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *
                "#,
                name,
                slug,
                parent_id,
                description,
                created_by
            ).fetch_one(pool).await?
        )
    }

    pub async fn update(pool: &PgPool, org: &Organization) -> Result<u64, Error> {
        Ok(
            sqlx::query!(
                r#"
                UPDATE organizations
                SET name = $2, slug = $3, parent_id = $4, description = $5, updated_at = NOW()
                WHERE id = $1 AND deleted_at IS NULL
                "#,
                org.id,
                org.name,
                org.slug,
                org.parent_id,
                org.description,
            ).execute(pool).await?.rows_affected()
        )
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(
            sqlx::query!(
                "UPDATE organizations SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
                id
            ).execute(pool).await?.rows_affected()
        )
    }
}
