use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationResponse {
    pub anilist_id: i32,
    pub title_romaji: String,
    pub title_english: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub cover_image_url: Option<String>,
    pub episodes: Option<i32>,
    pub description: Option<String>,
}
