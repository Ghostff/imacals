use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct PermissionRepository;

impl PermissionRepository {
    #[allow(dead_code)]
    pub async fn can(pool: &PgPool, user_id: &Uuid, permission: &str) -> Result<bool, Error> {
        match sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM user_permissions up
                JOIN permissions p ON p.id = up.permission_id
                WHERE up.user_id = $1 AND up.deleted_at IS NULL AND p.name = $2
                LIMIT 1
            )
            "#,
            user_id,
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
    pub async fn can_any(pool: &PgPool, user_id: &Uuid, permissions: Vec<&str>) -> Result<bool, Error> {
        let permissions: Vec<String> = permissions.into_iter().map(String::from).collect();

        match sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM user_permissions up
                JOIN permissions p ON p.id = up.permission_id
                WHERE up.user_id = $1 AND up.deleted_at IS NULL AND p.name = ANY($2)
                LIMIT 1
            )
            "#,
            user_id,
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

    // True only when every requested permission is held. Counts DISTINCT names so a duplicate in
    // the caller's list cannot make a partial grant look complete.
    #[allow(dead_code)]
    pub async fn can_all(pool: &PgPool, user_id: &Uuid, permissions: Vec<&str>) -> Result<bool, Error> {
        let mut permissions: Vec<String> = permissions.into_iter().map(String::from).collect();
        permissions.sort();
        permissions.dedup();
        let required_count = permissions.len() as i64;

        match sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT p.name)
            FROM user_permissions up
            JOIN permissions p ON p.id = up.permission_id
            WHERE up.user_id = $1 AND up.deleted_at IS NULL AND p.name = ANY($2)
            "#,
            user_id,
            &permissions,
        )
        .fetch_one(pool)
        .await {
            Ok(Some(found_count)) => Ok(found_count == required_count),
            Ok(None) => Ok(required_count == 0),
            Err(e) => Err(e),
        }
    }

    // Replaces a user's grants with exactly the permissions a role bundles. Used when assigning or
    // re-syncing a role.
    #[allow(dead_code)]
    pub async fn sync_from_role(pool: &PgPool, user_id: &Uuid, role_id: &Uuid) -> Result<u64, Error> {
        let mut tx = pool.begin().await?;

        sqlx::query!(
            "UPDATE user_permissions SET deleted_at = NOW() WHERE user_id = $1 AND deleted_at IS NULL",
            user_id
        )
        .execute(&mut *tx)
        .await?;

        let inserted = sqlx::query!(
            r#"
            INSERT INTO user_permissions (user_id, permission_id)
            SELECT $1, rp.permission_id
            FROM role_permissions rp
            WHERE rp.role_id = $2
            ON CONFLICT (user_id, permission_id) WHERE deleted_at IS NULL DO NOTHING
            "#,
            user_id,
            role_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        tx.commit().await?;
        Ok(inserted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    async fn make_user(pool: &PgPool, email: &str) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T',$1,'x',NOW()) RETURNING id",
            email
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn grant(pool: &PgPool, user_id: &Uuid, permission: &str) {
        sqlx::query!(
            "INSERT INTO user_permissions (user_id, permission_id)
             SELECT $1, id FROM permissions WHERE name = $2",
            user_id,
            permission
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn can_is_false_without_a_grant(pool: PgPool) {
        let user_id = make_user(&pool, "nogrant@test.com").await;
        assert!(!PermissionRepository::can(&pool, &user_id, "users.view").await.unwrap());
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn can_is_true_after_a_grant(pool: PgPool) {
        let user_id = make_user(&pool, "granted@test.com").await;
        grant(&pool, &user_id, "users.view").await;
        assert!(PermissionRepository::can(&pool, &user_id, "users.view").await.unwrap());
    }

    // A soft-deleted grant must not still authorise the user.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn soft_deleted_grant_does_not_authorise(pool: PgPool) {
        let user_id = make_user(&pool, "revoked@test.com").await;
        grant(&pool, &user_id, "users.view").await;
        sqlx::query!("UPDATE user_permissions SET deleted_at = NOW() WHERE user_id = $1", user_id)
            .execute(&pool)
            .await
            .unwrap();

        assert!(!PermissionRepository::can(&pool, &user_id, "users.view").await.unwrap());
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn can_any_matches_a_single_held_permission(pool: PgPool) {
        let user_id = make_user(&pool, "any@test.com").await;
        grant(&pool, &user_id, "users.view").await;

        assert!(PermissionRepository::can_any(&pool, &user_id, vec!["users.delete", "users.view"]).await.unwrap());
        assert!(!PermissionRepository::can_any(&pool, &user_id, vec!["users.delete"]).await.unwrap());
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn can_all_requires_every_permission(pool: PgPool) {
        let user_id = make_user(&pool, "all@test.com").await;
        grant(&pool, &user_id, "users.view").await;

        assert!(PermissionRepository::can_all(&pool, &user_id, vec!["users.view"]).await.unwrap());
        assert!(!PermissionRepository::can_all(&pool, &user_id, vec!["users.view", "users.delete"]).await.unwrap());
    }

    // A duplicate in the caller's list must not inflate the match count into a false pass.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn can_all_ignores_duplicates_in_the_request(pool: PgPool) {
        let user_id = make_user(&pool, "dupes@test.com").await;
        grant(&pool, &user_id, "users.view").await;

        assert!(PermissionRepository::can_all(&pool, &user_id, vec!["users.view", "users.view"]).await.unwrap());
        assert!(!PermissionRepository::can_all(&pool, &user_id, vec!["users.view", "users.view", "users.delete"]).await.unwrap());
    }

    // Soft-deleting the user cascades to their grants via the migration's trigger.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleting_a_user_revokes_their_grants(pool: PgPool) {
        let user_id = make_user(&pool, "cascade@test.com").await;
        grant(&pool, &user_id, "users.view").await;
        sqlx::query!("UPDATE users SET deleted_at = NOW() WHERE id = $1", user_id)
            .execute(&pool)
            .await
            .unwrap();

        let live: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_permissions WHERE user_id = $1 AND deleted_at IS NULL",
            user_id
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap_or(0);

        assert_eq!(live, 0, "grants should follow the user into soft-delete");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn sync_from_role_grants_the_roles_bundle(pool: PgPool) {
        let user_id = make_user(&pool, "sync@test.com").await;
        let role_id: Uuid = sqlx::query_scalar!("SELECT id FROM roles WHERE name = 'order-desk'")
            .fetch_one(&pool)
            .await
            .unwrap();

        PermissionRepository::sync_from_role(&pool, &user_id, &role_id).await.unwrap();

        assert!(PermissionRepository::can(&pool, &user_id, "users.view").await.unwrap());
        assert!(!PermissionRepository::can(&pool, &user_id, "users.delete").await.unwrap());
    }

    // Re-syncing to a different role must drop the permissions the old role granted.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn sync_from_role_replaces_previous_grants(pool: PgPool) {
        let user_id = make_user(&pool, "resync@test.com").await;
        let admin_id: Uuid = sqlx::query_scalar!("SELECT id FROM roles WHERE name = 'admin'")
            .fetch_one(&pool).await.unwrap();
        let warehouse_id: Uuid = sqlx::query_scalar!("SELECT id FROM roles WHERE name = 'warehouse'")
            .fetch_one(&pool).await.unwrap();

        PermissionRepository::sync_from_role(&pool, &user_id, &admin_id).await.unwrap();
        assert!(PermissionRepository::can(&pool, &user_id, "users.delete").await.unwrap());

        // Warehouse currently bundles nothing, so the admin grants must be gone with none added.
        PermissionRepository::sync_from_role(&pool, &user_id, &warehouse_id).await.unwrap();
        assert!(!PermissionRepository::can(&pool, &user_id, "users.delete").await.unwrap());
        assert!(!PermissionRepository::can(&pool, &user_id, "users.view").await.unwrap());
    }
}
