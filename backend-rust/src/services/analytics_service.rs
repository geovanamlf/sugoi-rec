use std::collections::HashMap;

use crate::{
    errors::AppError,
    repositories::analytics_repository,
    schemas::analytics::{GenreStatsResponse, RatingStatsResponse, StatusStatsResponse},
};

pub async fn get_genre_stats(
    db: &sqlx::PgPool,
    user_id: i32,
) -> Result<Vec<GenreStatsResponse>, AppError> {
    let genre_rows = analytics_repository::get_genres_by_user(db, user_id).await?;
    let mut counts: HashMap<String, i64> = HashMap::new();

    for genres in genre_rows.into_iter().flatten() {
        for genre in genres.split(',') {
            let genre = genre.trim();

            if genre.is_empty() {
                continue;
            }

            *counts.entry(genre.to_string()).or_insert(0) += 1;
        }
    }

    let mut result = counts
        .into_iter()
        .map(|(genre, count)| GenreStatsResponse { genre, count })
        .collect::<Vec<_>>();

    result.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.genre.cmp(&right.genre))
    });

    Ok(result)
}

pub async fn get_rating_stats(
    db: &sqlx::PgPool,
    user_id: i32,
) -> Result<RatingStatsResponse, AppError> {
    let row = analytics_repository::get_rating_stats(db, user_id).await?;

    Ok(RatingStatsResponse {
        average_rating: row.average_rating,
        rated_count: row.rated_count,
    })
}

pub async fn get_status_stats(
    db: &sqlx::PgPool,
    user_id: i32,
) -> Result<Vec<StatusStatsResponse>, AppError> {
    let statuses = analytics_repository::get_statuses_by_user(db, user_id).await?;
    let mut counts: HashMap<String, i64> = HashMap::new();

    for status in statuses {
        *counts.entry(status).or_insert(0) += 1;
    }

    let mut result = counts
        .into_iter()
        .map(|(status, count)| StatusStatsResponse { status, count })
        .collect::<Vec<_>>();

    result.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.status.cmp(&right.status))
    });

    Ok(result)
}
