use chrono::NaiveDateTime;
use sqlx::PgPool;

use crate::models::user_anime::{UserAnime, UserAnimeWithAnime};

pub async fn anime_exists(db: &PgPool, anime_id: i32) -> Result<bool, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM anime_cache
            WHERE id = $1
        )
        "#,
    )
    .bind(anime_id)
    .fetch_one(db)
    .await?;

    Ok(exists)
}

pub async fn find_by_user_and_anime(
    db: &PgPool,
    user_id: i32,
    anime_id: i32,
) -> Result<Option<UserAnime>, sqlx::Error> {
    sqlx::query_as::<_, UserAnime>(
        r#"
        SELECT id, user_id, anime_id, status, rating, is_favorite, added_at
        FROM user_anime
        WHERE user_id = $1 AND anime_id = $2
        "#,
    )
    .bind(user_id)
    .bind(anime_id)
    .fetch_optional(db)
    .await
}

pub async fn list_by_user(
    db: &PgPool,
    user_id: i32,
) -> Result<Vec<UserAnimeWithAnime>, sqlx::Error> {
    sqlx::query_as::<_, UserAnimeWithAnime>(
        r#"
        SELECT
            ua.id,
            ua.user_id,
            ua.anime_id,
            ua.status,
            ua.rating,
            ua.is_favorite,
            ua.added_at,

            a.anilist_id AS anime_anilist_id,
            a.title_romaji AS anime_title_romaji,
            a.title_english AS anime_title_english,
            a.cover_image_url AS anime_cover_image_url,
            a.episode_count AS anime_episode_count,
            a.genres AS anime_genres
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

pub async fn create(
    db: &PgPool,
    user_id: i32,
    anime_id: i32,
    status: &str,
    rating: Option<i32>,
    is_favorite: bool,
    added_at: NaiveDateTime,
) -> Result<UserAnime, sqlx::Error> {
    sqlx::query_as::<_, UserAnime>(
        r#"
        INSERT INTO user_anime (
            user_id,
            anime_id,
            status,
            rating,
            is_favorite,
            added_at
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, anime_id, status, rating, is_favorite, added_at
        "#,
    )
    .bind(user_id)
    .bind(anime_id)
    .bind(status)
    .bind(rating)
    .bind(is_favorite)
    .bind(added_at)
    .fetch_one(db)
    .await
}

pub async fn update(
    db: &PgPool,
    id: i32,
    status: &str,
    rating: Option<i32>,
    is_favorite: bool,
) -> Result<UserAnime, sqlx::Error> {
    sqlx::query_as::<_, UserAnime>(
        r#"
        UPDATE user_anime
        SET
            status = $1,
            rating = $2,
            is_favorite = $3
        WHERE id = $4
        RETURNING id, user_id, anime_id, status, rating, is_favorite, added_at
        "#,
    )
    .bind(status)
    .bind(rating)
    .bind(is_favorite)
    .bind(id)
    .fetch_one(db)
    .await
}

pub async fn delete_by_id(db: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM user_anime
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(db)
    .await?;

    Ok(())
}
