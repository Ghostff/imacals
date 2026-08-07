use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::role::{Role, RoleWithPermissions};

pub struct RoleRepository;

impl RoleRepository {
    pub async fn get_organization_roles_with_permissions(pool: &PgPool, organization_id: &Uuid) -> Result<Vec<RoleWithPermissions>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                r.id, r.name, r.title, r.description, r.organization_id, r.created_at, r.updated_at, r.deleted_at,
                COALESCE(json_agg(json_build_object(
                    'id', p.id,
                    'name', p.name
                )) FILTER (WHERE p.id IS NOT NULL), '[]') AS "permissions!: serde_json::Value"
            FROM roles r
            LEFT JOIN role_permissions rp ON r.id = rp.role_id
            LEFT JOIN permissions p ON rp.permission_id = p.id
            WHERE (r.organization_id = $1 OR r.organization_id IS NULL) AND r.deleted_at IS NULL
            GROUP BY r.id
            "#,
            organization_id
        ).fetch_all(pool).await?;

        Ok(rows.into_iter().map(|row| RoleWithPermissions {
            role: Role {
                id: row.id,
                name: row.name,
                title: row.title,
                description: row.description,
                organization_id: row.organization_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            },
            permissions: serde_json::from_value(row.permissions).unwrap_or_default(),
        }).collect())
    }

    pub async fn get_organization_roles(pool: &PgPool, organization_id: &Uuid) -> Result<Vec<Role>, Error> {
        // Include global roles (organization_id IS NULL) alongside org-specific ones.
        Ok(sqlx::query_as!(
            Role,
            "SELECT * FROM roles WHERE (organization_id = $1 OR organization_id IS NULL) AND deleted_at IS NULL ORDER BY title",
            organization_id
        ).fetch_all(pool).await?)
    }

}