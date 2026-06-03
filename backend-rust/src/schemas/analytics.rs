use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GenreStatsResponse {
    pub genre: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct RatingStatsResponse {
    pub average_rating: Option<f64>,
    pub rated_count: i64,
}

#[derive(Debug, Serialize)]
pub struct StatusStatsResponse {
    pub status: String,
    pub count: i64,
}
