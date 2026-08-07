use crate::models::user_bank_account::{CreateUserBankAccountSchema, UserBankAccount};
use sqlx::{Error, PgPool};
use uuid::Uuid;

pub struct UserBankAccountRepository;

impl UserBankAccountRepository {
    pub async fn get_for_user(pool: &PgPool, user_id: &Uuid) -> Result<Vec<UserBankAccount>, Error> {
        Ok(sqlx::query_as!(
            UserBankAccount,
            "SELECT * FROM user_bank_accounts WHERE user_id = $1 AND deleted_at IS NULL ORDER BY is_primary DESC, created_at",
            user_id
        ).fetch_all(pool).await?)
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid, user_id: &Uuid) -> Result<UserBankAccount, Error> {
        Ok(sqlx::query_as!(
            UserBankAccount,
            "SELECT * FROM user_bank_accounts WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL LIMIT 1",
            id, user_id
        ).fetch_one(pool).await?)
    }

    pub async fn create(pool: &PgPool, user_id: &Uuid, body: &CreateUserBankAccountSchema) -> Result<UserBankAccount, Error> {
        let account_type = body.account_type.as_deref().unwrap_or("checking");
        Ok(sqlx::query_as!(
            UserBankAccount,
            r#"
            INSERT INTO user_bank_accounts (user_id, bank_name, account_holder_name, account_type, account_number, routing_number, is_primary)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            user_id,
            body.bank_name,
            body.account_holder_name,
            account_type,
            body.account_number,
            body.routing_number,
            body.is_primary,
        ).fetch_one(pool).await?)
    }

    pub async fn update(pool: &PgPool, id: &Uuid, user_id: &Uuid, body: &CreateUserBankAccountSchema) -> Result<u64, Error> {
        let account_type = body.account_type.as_deref().unwrap_or("checking");
        Ok(sqlx::query!(
            r#"
            UPDATE user_bank_accounts SET
                bank_name           = $3,
                account_holder_name = $4,
                account_type        = $5,
                account_number      = $6,
                routing_number      = $7,
                is_primary          = $8,
                updated_at          = NOW()
            WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
            "#,
            id, user_id,
            body.bank_name,
            body.account_holder_name,
            account_type,
            body.account_number,
            body.routing_number,
            body.is_primary,
        ).execute(pool).await?.rows_affected())
    }

    pub async fn delete(pool: &PgPool, id: &Uuid, user_id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE user_bank_accounts SET deleted_at = NOW() WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
            id, user_id
        ).execute(pool).await?.rows_affected())
    }
}
