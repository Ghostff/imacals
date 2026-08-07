use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::models::attribute::{Attribute, CreateAttributeSchema, UpdateAttributeSchema};

// AttributeRepository is the only place that talks to the attributes table.
pub struct AttributeRepository;

impl AttributeRepository {
    // Returns all active attributes for a given owner — the primary query for reading integration credentials.
    pub async fn find_for_owner(
        pool: &PgPool,
        attributeable_type: &str,
        attributeable_id: &Uuid,
    ) -> Result<Vec<Attribute>, Error> {
        Ok(sqlx::query_as!(
            Attribute,
            r#"SELECT id, created_by, attributeable_type, attributeable_id,
                      name, value, type AS attribute_type, is_encrypted,
                      created_at, updated_at, deleted_at
               FROM attributes
               WHERE attributeable_type = $1
                 AND attributeable_id   = $2
                 AND deleted_at IS NULL
               ORDER BY name ASC"#,
            attributeable_type,
            attributeable_id,
        )
        .fetch_all(pool)
        .await?)
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Attribute, Error> {
        Ok(sqlx::query_as!(
            Attribute,
            r#"SELECT id, created_by, attributeable_type, attributeable_id,
                      name, value, type AS attribute_type, is_encrypted,
                      created_at, updated_at, deleted_at
               FROM attributes
               WHERE id = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            id,
        )
        .fetch_one(pool)
        .await?)
    }

    // Accepts any PgExecutor so it can be called within a transaction from the service layer.
    pub async fn create<'e>(
        executor: impl sqlx::PgExecutor<'e>,
        created_by: &Uuid,
        schema: &CreateAttributeSchema,
    ) -> Result<Attribute, Error> {
        Ok(sqlx::query_as!(
            Attribute,
            r#"INSERT INTO attributes
                   (created_by, attributeable_type, attributeable_id, name, value, type, is_encrypted)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, created_by, attributeable_type, attributeable_id,
                         name, value, type AS attribute_type, is_encrypted,
                         created_at, updated_at, deleted_at"#,
            created_by,
            schema.attributeable_type,
            schema.attributeable_id,
            schema.name,
            schema.value,
            schema.attribute_type,
            schema.is_encrypted.unwrap_or(false),
        )
        .fetch_one(executor)
        .await?)
    }

    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        schema: &UpdateAttributeSchema,
    ) -> Result<Attribute, Error> {
        Ok(sqlx::query_as!(
            Attribute,
            r#"UPDATE attributes
               SET name         = COALESCE($2, name),
                   value        = COALESCE($3, value),
                   type         = COALESCE($4, type),
                   is_encrypted = COALESCE($5, is_encrypted),
                   updated_at   = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, created_by, attributeable_type, attributeable_id,
                         name, value, type AS attribute_type, is_encrypted,
                         created_at, updated_at, deleted_at"#,
            id,
            schema.name,
            schema.value,
            schema.attribute_type,
            schema.is_encrypted,
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE attributes SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(pool)
        .await?
        .rows_affected())
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

    fn make_schema(owner_id: Uuid, name: &str) -> CreateAttributeSchema {
        CreateAttributeSchema {
            attributeable_type: "integrations".into(),
            attributeable_id: owner_id,
            name: name.into(),
            value: Some("https://rets.example.com".into()),
            attribute_type: "url".into(),
            is_encrypted: None,
        }
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn created_attribute_appears_for_owner(pool: PgPool) {
        let user_id  = make_user(&pool, "attr-owner@test.com").await;
        let owner_id = Uuid::new_v4();
        AttributeRepository::create(&pool, &user_id, &make_schema(owner_id, "url")).await.unwrap();
        let rows = AttributeRepository::find_for_owner(&pool, "integrations", &owner_id).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "url");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn find_for_owner_only_returns_matching_owner(pool: PgPool) {
        let user_id   = make_user(&pool, "attr-scope@test.com").await;
        let owner_a   = Uuid::new_v4();
        let owner_b   = Uuid::new_v4();
        AttributeRepository::create(&pool, &user_id, &make_schema(owner_a, "url-a")).await.unwrap();
        AttributeRepository::create(&pool, &user_id, &make_schema(owner_b, "url-b")).await.unwrap();
        let rows = AttributeRepository::find_for_owner(&pool, "integrations", &owner_a).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "url-a");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleted_attribute_is_hidden(pool: PgPool) {
        let user_id  = make_user(&pool, "attr-del@test.com").await;
        let owner_id = Uuid::new_v4();
        let attr = AttributeRepository::create(&pool, &user_id, &make_schema(owner_id, "url")).await.unwrap();
        AttributeRepository::delete(&pool, &attr.id).await.unwrap();
        let rows = AttributeRepository::find_for_owner(&pool, "integrations", &owner_id).await.unwrap();
        assert!(rows.is_empty(), "soft-deleted attribute must not appear");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn double_delete_returns_zero(pool: PgPool) {
        let user_id  = make_user(&pool, "attr-dup@test.com").await;
        let owner_id = Uuid::new_v4();
        let attr = AttributeRepository::create(&pool, &user_id, &make_schema(owner_id, "url")).await.unwrap();
        AttributeRepository::delete(&pool, &attr.id).await.unwrap();
        let affected = AttributeRepository::delete(&pool, &attr.id).await.unwrap();
        assert_eq!(affected, 0);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn update_changes_value(pool: PgPool) {
        let user_id  = make_user(&pool, "attr-upd@test.com").await;
        let owner_id = Uuid::new_v4();
        let attr = AttributeRepository::create(&pool, &user_id, &make_schema(owner_id, "url")).await.unwrap();
        let schema = UpdateAttributeSchema {
            name: None,
            value: Some("https://new-rets.example.com".into()),
            attribute_type: None,
            is_encrypted: None,
        };
        let updated = AttributeRepository::update(&pool, &attr.id, &schema).await.unwrap();
        assert_eq!(updated.value.as_deref(), Some("https://new-rets.example.com"));
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn find_by_id_after_delete_is_not_found(pool: PgPool) {
        let user_id  = make_user(&pool, "attr-gone@test.com").await;
        let owner_id = Uuid::new_v4();
        let attr = AttributeRepository::create(&pool, &user_id, &make_schema(owner_id, "url")).await.unwrap();
        AttributeRepository::delete(&pool, &attr.id).await.unwrap();
        assert!(matches!(AttributeRepository::find_by_id(&pool, &attr.id).await, Err(sqlx::Error::RowNotFound)));
    }
}
