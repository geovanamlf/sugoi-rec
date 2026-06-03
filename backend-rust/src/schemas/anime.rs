use chrono::NaiveDateTime;
use serde::Serialize;

use crate::models::anime::Anime;

#[derive(Debug, Clone, Serialize)]
pub struct AnimeResponse {
    pub id: i32,
    pub anilist_id: i32,
    pub title_romaji: String,
    pub title_english: Option<String>,
    pub title_native: Option<String>,
    pub episode_count: Option<i32>,
    pub cover_image_url: Option<String>,
    pub description: Option<String>,
    pub genres: Option<String>,
    pub tags: Option<String>,
    pub demographic: Option<String>,
    pub cached_at: NaiveDateTime,
}

impl From<Anime> for AnimeResponse {
    fn from(anime: Anime) -> Self {
        Self {
            id: anime.id,
            anilist_id: anime.anilist_id,
            title_romaji: anime.title_romaji,
            title_english: anime.title_english,
            title_native: anime.title_native,
            episode_count: anime.episode_count,
            cover_image_url: anime.cover_image_url,
            description: anime.description,
            genres: anime.genres,
            tags: anime.tags,
            demographic: anime.demographic,
            cached_at: anime.cached_at,
        }
    }
}
