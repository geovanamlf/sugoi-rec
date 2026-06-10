use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserAnime {
    pub id: i32,
    pub user_id: i32,
    pub anime_id: i32,
    pub status: String,
    pub rating: Option<i32>,
    pub is_favorite: bool,
    pub added_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserAnimeWithAnime {
    pub id: i32,
    pub user_id: i32,
    pub anime_id: i32,
    pub status: String,
    pub rating: Option<i32>,
    pub is_favorite: bool,
    pub added_at: NaiveDateTime,

    pub anime_anilist_id: i32,
    pub anime_title_romaji: String,
    pub anime_title_english: Option<String>,
    pub anime_cover_image_url: Option<String>,
    pub anime_episode_count: Option<i32>,
    pub anime_genres: Option<String>,
}
