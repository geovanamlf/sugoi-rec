use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize};

use crate::models::user_anime::{UserAnime, UserAnimeWithAnime};

#[derive(Debug, Deserialize)]
pub struct UserAnimeCreate {
    pub anime_id: i32,
    pub status: String,
    pub rating: Option<i32>,

    #[serde(default)]
    pub is_favorite: bool,
}

#[derive(Debug, Deserialize)]
pub struct UserAnimeUpdate {
    #[serde(default)]
    pub status: Option<String>,

    #[serde(default, deserialize_with = "deserialize_nullable_rating")]
    pub rating: Option<Option<i32>>,

    #[serde(default)]
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UserAnimeListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct UserAnimeResponse {
    pub id: i32,
    pub anime_id: i32,
    pub user_id: i32,
    pub status: String,
    pub rating: Option<i32>,
    pub is_favorite: bool,
    pub added_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct UserAnimeListResponse {
    pub items: Vec<UserAnimeListItemResponse>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct UserAnimeListItemResponse {
    pub id: i32,
    pub anime_id: i32,
    pub user_id: i32,
    pub status: String,
    pub rating: Option<i32>,
    pub is_favorite: bool,
    pub added_at: NaiveDateTime,
    pub anime: AnimePreviewResponse,
}

#[derive(Debug, Serialize)]
pub struct AnimePreviewResponse {
    pub id: i32,
    pub anilist_id: i32,
    pub title_romaji: String,
    pub title_english: Option<String>,
    pub cover_image_url: Option<String>,
    pub episode_count: Option<i32>,
    pub genres: Option<String>,
}

impl From<UserAnime> for UserAnimeResponse {
    fn from(entry: UserAnime) -> Self {
        Self {
            id: entry.id,
            anime_id: entry.anime_id,
            user_id: entry.user_id,
            status: entry.status,
            rating: entry.rating,
            is_favorite: entry.is_favorite,
            added_at: entry.added_at,
        }
    }
}

impl From<UserAnimeWithAnime> for UserAnimeListItemResponse {
    fn from(row: UserAnimeWithAnime) -> Self {
        Self {
            id: row.id,
            anime_id: row.anime_id,
            user_id: row.user_id,
            status: row.status,
            rating: row.rating,
            is_favorite: row.is_favorite,
            added_at: row.added_at,
            anime: AnimePreviewResponse {
                id: row.anime_id,
                anilist_id: row.anime_anilist_id,
                title_romaji: row.anime_title_romaji,
                title_english: row.anime_title_english,
                cover_image_url: row.anime_cover_image_url,
                episode_count: row.anime_episode_count,
                genres: row.anime_genres,
            },
        }
    }
}

fn deserialize_nullable_rating<'de, D>(deserializer: D) -> Result<Option<Option<i32>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<i32>::deserialize(deserializer).map(Some)
}
