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
    _existing_id: Option<i32>,
    data: &ParsedAnimeData,
) -> Result<Anime, sqlx::Error> {
    upsert_by_anilist_id(db, data).await
}

async fn upsert_by_anilist_id(db: &PgPool, data: &ParsedAnimeData) -> Result<Anime, sqlx::Error> {
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
        ON CONFLICT (anilist_id)
        DO UPDATE SET
            title_romaji = EXCLUDED.title_romaji,
            title_english = EXCLUDED.title_english,
            title_native = EXCLUDED.title_native,
            episode_count = EXCLUDED.episode_count,
            cover_image_url = EXCLUDED.cover_image_url,
            description = EXCLUDED.description,
            genres = EXCLUDED.genres,
            tags = EXCLUDED.tags,
            demographic = EXCLUDED.demographic,
            cached_at = EXCLUDED.cached_at
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
