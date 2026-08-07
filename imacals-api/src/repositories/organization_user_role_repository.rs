use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::organization_user_role::OrganizationUserRole;

pub struct OrganizationUserRoleRepository;

impl OrganizationUserRoleRepository {
    // Returns global job-title roles plus any org-specific ones, sorted by title.
    pub async fn index(pool: &PgPool, organization_id: &Uuid) -> Result<Vec<OrganizationUserRole>, Error> {
        Ok(sqlx::query_as!(
            OrganizationUserRole,
            "SELECT * FROM organization_user_role
             WHERE (organization_id = $1 OR organization_id IS NULL) AND deleted_at IS NULL
             ORDER BY title",
            organization_id
        ).fetch_all(pool).await?)
    }

    // Returns the subset of global job-title roles that may be used as system users.
    pub async fn system_user_eligible(pool: &PgPool) -> Result<Vec<OrganizationUserRole>, Error> {
        Ok(sqlx::query_as!(
            OrganizationUserRole,
            "SELECT * FROM organization_user_role
             WHERE system_user_eligible = TRUE AND organization_id IS NULL AND deleted_at IS NULL
             ORDER BY title"
        ).fetch_all(pool).await?)
    }

    // Resolves the job-title role assigned to a user in a specific org.
    pub async fn get_user_role_for_user(
        pool: &PgPool,
        user_id: &Uuid,
        organization_id: &Uuid,
    ) -> Result<Option<OrganizationUserRole>, Error> {
        Ok(sqlx::query_as!(
            OrganizationUserRole,
            r#"
            SELECT ur.*
            FROM organization_user_role ur
            JOIN organization_users ou ON ou.user_role_id = ur.id
            WHERE ou.user_id = $1
              AND ou.organization_id = $2
              AND ou.deleted_at IS NULL
              AND ur.deleted_at IS NULL
            LIMIT 1
            "#,
            user_id,
            organization_id,
        ).fetch_optional(pool).await?)
    }
}
