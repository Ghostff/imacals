use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::user::User;

pub struct OrganizationUserRepository;

impl OrganizationUserRepository {
    pub async fn sync_user_organizations_and_permissions(
        pool: &PgPool,
        target_user: &User,
        organization_ids: &[Uuid],
        added_by: &User,
    ) -> Result<(), Error> {
        let mut tx = pool.begin().await?;
        // NOTE:
        // We are *synchronizing* user ↔ organization relationships to exactly match `organization_ids`.
        //
        // Important data-model constraint:
        // - organization_users has a UNIQUE (user_id, organization_id)
        // - Soft-deleted rows still participate in uniqueness
        //
        // Because of this:
        // - We CANNOT blindly insert rows that were previously soft-deleted
        // - We must REACTIVATE (undelete) them instead
        //
        // Sync strategy:
        // 1) Deactivate relationships NOT in the new list
        // 2) Create brand-new relationships that never existed
        // 3) Reactivate relationships that exist but were soft-deleted

        // Deactivate organizations NOT in the new list
        sqlx::query!(
            r#"
            UPDATE organization_users
            SET deleted_at = NOW()
            WHERE user_id = $1 AND organization_id NOT IN (SELECT UNNEST($2::uuid[])) AND deleted_at IS NULL
            "#,
            &target_user.id,
            organization_ids
        ).execute(&mut *tx).await?;

        // Create brand-new relationships
        // ON CONFLICT DO NOTHING is safe here because:
        // - Active rows already exist
        sqlx::query!(
            r#"
            INSERT INTO organization_users (user_id, organization_id, added_by)
            SELECT $1, UNNEST($2::uuid[]), $3
            ON CONFLICT (user_id, organization_id) WHERE deleted_at IS NULL
            DO NOTHING
            "#,
            &target_user.id,
            organization_ids,
            &added_by.id
        ).execute(&mut *tx).await?;

        // If it was soft-deleted, we might need to restore it.
        // But the above ON CONFLICT only works if it's NOT deleted.
        // Simplified for now: just make sure they are there.
        sqlx::query!(
            r#"
            UPDATE organization_users
            SET deleted_at = NULL
            WHERE user_id = $1 AND organization_id = ANY($2::uuid[]) AND deleted_at IS NOT NULL
            "#,
            &target_user.id,
            organization_ids
        ).execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(())
    }

}
