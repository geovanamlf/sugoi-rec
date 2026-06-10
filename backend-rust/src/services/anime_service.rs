use chrono::{Duration, Utc};

use crate::{
    app_state::AppState, clients::anilist_client, errors::AppError, models::anime::Anime,
    repositories::anime_repository, schemas::anime::AnimeResponse,
};

const CACHE_TTL_DAYS: i64 = 7;

pub async fn get_anime_by_anilist_id(
    state: &AppState,
    anilist_id: i32,
) -> Result<AnimeResponse, AppError> {
    if anilist_id <= 0 {
        return Err(AppError::BadRequest(
            "AniList ID must be a positive integer.".to_string(),
        ));
    }

    let cached = anime_repository::find_by_anilist_id(&state.db, anilist_id).await?;

    if let Some(anime) = cached {
        if is_cache_valid(&anime) {
            return Ok(AnimeResponse::from(anime));
        }

        let data = anilist_client::fetch_anime_by_id(
            &state.http_client,
            &state.config.anilist_url,
            anilist_id,
        )
        .await?;

        let saved = anime_repository::save_or_update(&state.db, Some(anime.id), &data).await?;
        return Ok(AnimeResponse::from(saved));
    }

    let data = anilist_client::fetch_anime_by_id(
        &state.http_client,
        &state.config.anilist_url,
        anilist_id,
    )
    .await?;

    let saved = anime_repository::save_or_update(&state.db, None, &data).await?;
    Ok(AnimeResponse::from(saved))
}

pub async fn get_anime_by_name(state: &AppState, search: &str) -> Result<AnimeResponse, AppError> {
    let search = search.trim();

    if search.is_empty() {
        return Err(AppError::BadRequest(
            "Search query is required.".to_string(),
        ));
    }

    let data =
        anilist_client::fetch_anime_by_name(&state.http_client, &state.config.anilist_url, search)
            .await?;

    let cached = anime_repository::find_by_anilist_id(&state.db, data.anilist_id).await?;
    let existing_id = cached.as_ref().map(|anime| anime.id);

    let saved = anime_repository::save_or_update(&state.db, existing_id, &data).await?;
    Ok(AnimeResponse::from(saved))
}

fn is_cache_valid(anime: &Anime) -> bool {
    let now = Utc::now().naive_utc();
    now - anime.cached_at < Duration::days(CACHE_TTL_DAYS)
}
