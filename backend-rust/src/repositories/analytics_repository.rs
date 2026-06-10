use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
pub struct RatingStatsRow {
    pub average_rating: Option<f64>,
    pub rated_count: i64,
}

pub async fn get_genres_by_user(
    db: &PgPool,
    user_id: i32,
) -> Result<Vec<Option<String>>, sqlx::Error> {
    let rows = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT a.genres
        FROM user_anime ua
        JOIN anime_cache a ON a.id = ua.anime_id
        WHERE ua.user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(rows)
}

pub async fn get_rating_stats(db: &PgPool, user_id: i32) -> Result<RatingStatsRow, sqlx::Error> {
    let row = sqlx::query_as::<_, RatingStatsRow>(
        r#"
        SELECT
            ROUND(AVG(rating)::numeric, 2)::float8 AS average_rating,
            COUNT(rating)::bigint AS rated_count
        FROM user_anime
        WHERE user_id = $1
          AND rating IS NOT NULL
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;

    Ok(row)
}

pub async fn get_statuses_by_user(db: &PgPool, user_id: i32) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT status
        FROM user_anime
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    Ok(rows)
}
