use sqlx::{Error, PgPool};
use crate::models::role::{Role, RoleWithPermissions};

pub struct RoleRepository;

impl RoleRepository {
    pub async fn index_with_permissions(pool: &PgPool) -> Result<Vec<RoleWithPermissions>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                r.id, r.name, r.title, r.description, r.created_at, r.updated_at, r.deleted_at,
                COALESCE(json_agg(json_build_object(
                    'id', p.id,
                    'name', p.name
                )) FILTER (WHERE p.id IS NOT NULL), '[]') AS "permissions!: serde_json::Value"
            FROM roles r
            LEFT JOIN role_permissions rp ON r.id = rp.role_id
            LEFT JOIN permissions p ON rp.permission_id = p.id
            WHERE r.deleted_at IS NULL
            GROUP BY r.id
            ORDER BY r.title
            "#
        ).fetch_all(pool).await?;

        Ok(rows.into_iter().map(|row| RoleWithPermissions {
            role: Role {
                id: row.id,
                name: row.name,
                title: row.title,
                description: row.description,
                created_at: row.created_at,
                updated_at: row.updated_at,
                deleted_at: row.deleted_at,
            },
            permissions: serde_json::from_value(row.permissions).unwrap_or_default(),
        }).collect())
    }

    #[allow(dead_code)]
    pub async fn index(pool: &PgPool) -> Result<Vec<Role>, Error> {
        Ok(sqlx::query_as!(
            Role,
            "SELECT * FROM roles WHERE deleted_at IS NULL ORDER BY title"
        ).fetch_all(pool).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // The seeded roles must come back, and admin must carry the full permission bundle.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn index_returns_seeded_roles(pool: PgPool) {
        let roles = RoleRepository::index(&pool).await.unwrap();
        let names: Vec<&str> = roles.iter().map(|r| r.name.as_str()).collect();

        assert!(names.contains(&"admin"));
        assert!(names.contains(&"order-desk"));
        assert!(names.contains(&"warehouse"));
        assert!(names.contains(&"dispatch"));
        assert!(names.contains(&"accounts"));
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn admin_role_has_every_permission(pool: PgPool) {
        let roles = RoleRepository::index_with_permissions(&pool).await.unwrap();
        let admin = roles.iter().find(|r| r.role.name == "admin").expect("admin role seeded");

        let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM permissions")
            .fetch_one(&pool).await.unwrap().unwrap_or(0);

        assert_eq!(admin.permissions.len() as i64, total);
    }

    // A soft-deleted role must drop out of the listing.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn soft_deleted_role_is_hidden(pool: PgPool) {
        sqlx::query!("UPDATE roles SET deleted_at = NOW() WHERE name = 'warehouse'")
            .execute(&pool).await.unwrap();

        let roles = RoleRepository::index(&pool).await.unwrap();
        assert!(!roles.iter().any(|r| r.name == "warehouse"));
    }
}
