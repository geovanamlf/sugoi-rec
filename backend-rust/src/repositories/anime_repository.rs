use sqlx::PgPool;

use crate::{clients::anilist_client::ParsedAnimeData, models::anime::Anime};

pub async fn find_by_anilist_id(
    db: &PgPool,
    anilist_id: i32,
) -> Result<Option<Anime>, sqlx::Error> {
    sqlx::query_as::<_, Anime>(
        r#"
        SELECT
            id,
            anilist_id,
            title_romaji,
            title_english,
            title_native,
            episode_count,
            cover_image_url,
            description,
            genres,
            tags,
            demographic,
            cached_at
        FROM anime_cache
        WHERE anilist_id = $1
        "#,
    )
    .bind(anilist_id)
    .fetch_optional(db)
    .await
}

pub async fn save_or_update(
    db: &PgPool,
    existing_id: Option<i32>,
    data: &ParsedAnimeData,
) -> Result<Anime, sqlx::Error> {
    match existing_id {
        Some(id) => update(db, id, data).await,
        None => insert(db, data).await,
    }
}

async fn insert(db: &PgPool, data: &ParsedAnimeData) -> Result<Anime, sqlx::Error> {
    sqlx::query_as::<_, Anime>(
        r#"
        INSERT INTO anime_cache (
            anilist_id,
            title_romaji,
            title_english,
            title_native,
            episode_count,
            cover_image_url,
            description,
            genres,
            tags,
            demographic,
            cached_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING
            id,
            anilist_id,
            title_romaji,
            title_english,
            title_native,
            episode_count,
            cover_image_url,
            description,
            genres,
            tags,
            demographic,
            cached_at
        "#,
    )
    .bind(data.anilist_id)
    .bind(&data.title_romaji)
    .bind(&data.title_english)
    .bind(&data.title_native)
    .bind(data.episode_count)
    .bind(&data.cover_image_url)
    .bind(&data.description)
    .bind(&data.genres)
    .bind(&data.tags)
    .bind(&data.demographic)
    .bind(data.cached_at)
    .fetch_one(db)
    .await
}

async fn update(db: &PgPool, id: i32, data: &ParsedAnimeData) -> Result<Anime, sqlx::Error> {
    sqlx::query_as::<_, Anime>(
        r#"
        UPDATE anime_cache
        SET
            anilist_id = $1,
            title_romaji = $2,
            title_english = $3,
            title_native = $4,
            episode_count = $5,
            cover_image_url = $6,
            description = $7,
            genres = $8,
            tags = $9,
            demographic = $10,
            cached_at = $11
        WHERE id = $12
        RETURNING
            id,
            anilist_id,
            title_romaji,
            title_english,
            title_native,
            episode_count,
            cover_image_url,
            description,
            genres,
            tags,
            demographic,
            cached_at
        "#,
    )
    .bind(data.anilist_id)
    .bind(&data.title_romaji)
    .bind(&data.title_english)
    .bind(&data.title_native)
    .bind(data.episode_count)
    .bind(&data.cover_image_url)
    .bind(&data.description)
    .bind(&data.genres)
    .bind(&data.tags)
    .bind(&data.demographic)
    .bind(data.cached_at)
    .bind(id)
    .fetch_one(db)
    .await
}
