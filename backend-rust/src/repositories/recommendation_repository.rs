use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserTasteAnime {
    pub status: String,
    pub rating: Option<i32>,
    pub is_favorite: bool,

    pub anilist_id: i32,
    pub genres: Option<String>,
    pub tags: Option<String>,
}

pub async fn list_user_taste_anime(
    db: &PgPool,
    user_id: i32,
) -> Result<Vec<UserTasteAnime>, sqlx::Error> {
    sqlx::query_as::<_, UserTasteAnime>(
        r#"
        SELECT
            ua.status,
            ua.rating,
            ua.is_favorite,

            a.anilist_id,
            a.genres,
            a.tags
        FROM user_anime ua
        JOIN anime_cache a ON a.id = ua.anime_id
        WHERE ua.user_id = $1
        ORDER BY ua.added_at DESC, ua.id DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn find_valid_recommendation_cache(
    db: &PgPool,
    user_id: i32,
    ttl_minutes: i32,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT data
        FROM recommendation_cache
        WHERE user_id = $1
          AND cached_at >= ((NOW() AT TIME ZONE 'UTC') - ($2::integer * INTERVAL '1 minute'))
        ORDER BY cached_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(ttl_minutes)
    .fetch_optional(db)
    .await
}

pub async fn find_latest_recommendation_cache(
    db: &PgPool,
    user_id: i32,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT data
        FROM recommendation_cache
        WHERE user_id = $1
        ORDER BY cached_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn replace_recommendation_cache(
    db: &PgPool,
    user_id: i32,
    data: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = db.begin().await?;

    sqlx::query(
        r#"
        DELETE FROM recommendation_cache
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO recommendation_cache (
            user_id,
            data,
            cached_at
        )
        VALUES ($1, $2, NOW() AT TIME ZONE 'UTC')
        "#,
    )
    .bind(user_id)
    .bind(data)
    .execute(&mut *tx)
    .await?;

    tx.commit().await
}

pub async fn delete_recommendation_cache(db: &PgPool, user_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM recommendation_cache
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .execute(db)
    .await?;

    Ok(())
}
