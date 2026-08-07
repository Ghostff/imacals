use sqlx::{PgPool, Error};
use uuid::Uuid;

pub struct OrganizationUserPermissionRepository;

impl OrganizationUserPermissionRepository {
    pub async fn sync_permissions(
        pool: &PgPool,
        organization_user_id: &Uuid,
        permission_ids: &[Uuid],
    ) -> Result<(), Error> {
        let mut tx = pool.begin().await?;

        // Remove existing permissions for this organization user
        sqlx::query!(
            "UPDATE organization_users_permissions SET deleted_at = NOW() WHERE organization_users_id = $1 AND deleted_at IS NULL",
            organization_user_id
        )
        .execute(&mut *tx)
        .await?;

        // Insert new permissions
        for permission_id in permission_ids {
            sqlx::query!(
                r#"
                INSERT INTO organization_users_permissions (organization_users_id, permission_id)
                VALUES ($1, $2)
                ON CONFLICT (organization_users_id, permission_id) WHERE deleted_at IS NULL
                DO UPDATE SET updated_at = NOW()
                "#,
                organization_user_id,
                permission_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
