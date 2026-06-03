use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Anime {
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
