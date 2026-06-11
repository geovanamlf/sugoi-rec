use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::refresh_token::RefreshToken;

pub async fn create(
    db: &PgPool,
    user_id: i32,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken, sqlx::Error> {
    let token_id = Uuid::new_v4();

    sqlx::query_as::<_, RefreshToken>(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            user_id,
            expires_at,
            revoked_at
        "#,
    )
    .bind(token_id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .fetch_one(db)
    .await
}

pub async fn create_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: i32,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken, sqlx::Error> {
    let token_id = Uuid::new_v4();

    sqlx::query_as::<_, RefreshToken>(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        RETURNING
            id,
            user_id,
            expires_at,
            revoked_at
        "#,
    )
    .bind(token_id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .fetch_one(&mut **transaction)
    .await
}

pub async fn find_by_hash_for_update(
    transaction: &mut Transaction<'_, Postgres>,
    token_hash: &str,
) -> Result<Option<RefreshToken>, sqlx::Error> {
    sqlx::query_as::<_, RefreshToken>(
        r#"
        SELECT
            id,
            user_id,
            expires_at,
            revoked_at
        FROM refresh_tokens
        WHERE token_hash = $1
        FOR UPDATE
        "#,
    )
    .bind(token_hash)
    .fetch_optional(&mut **transaction)
    .await
}

pub async fn revoke_by_id(
    transaction: &mut Transaction<'_, Postgres>,
    token_id: Uuid,
    replaced_by_token_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET
            revoked_at = COALESCE(revoked_at, now()),
            replaced_by_token_id = COALESCE($2, replaced_by_token_id)
        WHERE id = $1
        "#,
    )
    .bind(token_id)
    .bind(replaced_by_token_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

pub async fn revoke_active_tokens_for_user(
    transaction: &mut Transaction<'_, Postgres>,
    user_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now()
        WHERE user_id = $1
          AND revoked_at IS NULL
        "#,
    )
    .bind(user_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
