use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct PermissionRepository;

impl PermissionRepository {
    #[allow(dead_code)]
    pub async fn can(pool: &PgPool, user_id: &Uuid, organization_id: &Uuid, permission: &str) -> Result<bool, Error> {
        // The query checks if the user has any role that grants the given permission for the specific organization.
        match sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM organization_users ou
                JOIN organization_users_permissions oup ON oup.organization_users_id = ou.id AND oup.deleted_at IS NULL
                JOIN permissions p ON p.id = oup.permission_id
                WHERE ou.user_id = $1 AND ou.organization_id = $2 AND ou.deleted_at IS NULL AND p.name = $3
                LIMIT 1
            )
            "#,
            user_id,
            organization_id,
            permission,
        )
            .fetch_one(pool)
            .await {
            Ok(Some(true)) => Ok(true),
            Ok(Some(false)) | Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    #[allow(dead_code)]
    pub async fn can_any(
        pool: &PgPool,
        user_id: &Uuid,
        organization_id: &Uuid,
        permissions: Vec<&str>,
    ) -> Result<bool, Error> {
        let permissions: Vec<String> = permissions.into_iter().map(String::from).collect();

        match sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM organization_users ou
            JOIN organization_users_permissions oup ON oup.organization_users_id = ou.id AND oup.deleted_at IS NULL
            JOIN permissions p ON p.id = oup.permission_id
            WHERE ou.user_id = $1 AND ou.organization_id = $2 AND ou.deleted_at IS NULL AND p.name = ANY($3)
            LIMIT 1
        )
        "#,
        user_id,
        organization_id,
        &permissions,
    )
            .fetch_one(pool)
            .await
        {
            Ok(Some(true)) => Ok(true),
            Ok(Some(false)) | Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    #[allow(dead_code)]
    pub async fn can_all(pool: &PgPool, user_id: &Uuid, organization_id: &Uuid, permissions: Vec<&str>) -> Result<bool, Error> {
        let required_count = permissions.len() as i64;
        let permissions: Vec<String> = permissions.into_iter().map(String::from).collect();

        match sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT p.name)
            FROM organization_users ou
            JOIN organization_users_permissions oup ON oup.organization_users_id = ou.id AND oup.deleted_at IS NULL
            JOIN permissions p ON p.id = oup.permission_id
            WHERE ou.user_id = $1 AND ou.organization_id = $2 AND ou.deleted_at IS NULL AND p.name = ANY($3)
            "#,
            user_id,
            organization_id,
            &permissions,
        )
        .fetch_one(pool)
        .await {
            Ok(Some(found_count)) => Ok(found_count == required_count),
            Ok(None) => Ok(required_count == 0),
            Err(e) => Err(e),
        }
    }
}
