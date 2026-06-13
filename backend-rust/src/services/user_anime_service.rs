use chrono::Utc;

use crate::{
    errors::AppError,
    repositories::{recommendation_repository, user_anime_repository},
    schemas::user_anime::{
        UserAnimeCreate, UserAnimeListItemResponse, UserAnimeListQuery, UserAnimeListResponse,
        UserAnimeResponse, UserAnimeUpdate,
    },
};

const VALID_STATUSES: [&str; 4] = ["watching", "completed", "dropped", "planned"];

const DEFAULT_LIST_LIMIT: i64 = 20;
const MAX_LIST_LIMIT: i64 = 100;

struct Pagination {
    limit: i64,
    offset: i64,
}

pub async fn add_anime(
    db: &sqlx::PgPool,
    user_id: i32,
    payload: UserAnimeCreate,
) -> Result<UserAnimeResponse, AppError> {
    validate_anime_id(payload.anime_id)?;
    validate_status(&payload.status)?;
    validate_rating(payload.rating)?;

    let anime_exists = user_anime_repository::anime_exists(db, payload.anime_id).await?;

    if !anime_exists {
        return Err(AppError::NotFound("Anime not found.".to_string()));
    }

    let existing =
        user_anime_repository::find_by_user_and_anime(db, user_id, payload.anime_id).await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("Anime already in list.".to_string()));
    }

    let entry = user_anime_repository::create(
        db,
        user_id,
        payload.anime_id,
        &payload.status,
        payload.rating,
        payload.is_favorite,
        Utc::now().naive_utc(),
    )
    .await?;

    invalidate_recommendation_cache(db, user_id).await;

    Ok(UserAnimeResponse::from(entry))
}

pub async fn list_anime(
    db: &sqlx::PgPool,
    user_id: i32,
    query: UserAnimeListQuery,
) -> Result<UserAnimeListResponse, AppError> {
    let pagination = validate_pagination(query.limit, query.offset)?;

    let rows =
        user_anime_repository::list_by_user(db, user_id, pagination.limit, pagination.offset)
            .await?;

    let total = user_anime_repository::count_by_user(db, user_id).await?;

    let items = rows
        .into_iter()
        .map(UserAnimeListItemResponse::from)
        .collect();

    Ok(UserAnimeListResponse {
        items,
        limit: pagination.limit,
        offset: pagination.offset,
        total,
    })
}

pub async fn update_anime(
    db: &sqlx::PgPool,
    user_id: i32,
    anime_id: i32,
    payload: UserAnimeUpdate,
) -> Result<UserAnimeResponse, AppError> {
    validate_anime_id(anime_id)?;

    if let Some(status) = payload.status.as_deref() {
        validate_status(status)?;
    }

    if let Some(rating) = payload.rating {
        validate_rating(rating)?;
    }

    let existing = user_anime_repository::find_by_user_and_anime(db, user_id, anime_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Anime not found in list.".to_string()))?;

    let status = payload.status.unwrap_or(existing.status);
    let rating = payload.rating.unwrap_or(existing.rating);
    let is_favorite = payload.is_favorite.unwrap_or(existing.is_favorite);

    let updated =
        user_anime_repository::update(db, existing.id, &status, rating, is_favorite).await?;

    invalidate_recommendation_cache(db, user_id).await;

    Ok(UserAnimeResponse::from(updated))
}

pub async fn remove_anime(db: &sqlx::PgPool, user_id: i32, anime_id: i32) -> Result<(), AppError> {
    validate_anime_id(anime_id)?;

    let existing = user_anime_repository::find_by_user_and_anime(db, user_id, anime_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Anime not found in list.".to_string()))?;

    user_anime_repository::delete_by_id(db, existing.id).await?;

    invalidate_recommendation_cache(db, user_id).await;

    Ok(())
}

async fn invalidate_recommendation_cache(db: &sqlx::PgPool, user_id: i32) {
    if let Err(error) = recommendation_repository::delete_recommendation_cache(db, user_id).await {
        tracing::warn!(
            "Could not invalidate recommendation cache after user anime list change. user_id={user_id}, error={error}"
        );
    }
}

fn validate_anime_id(anime_id: i32) -> Result<(), AppError> {
    if anime_id <= 0 {
        return Err(AppError::BadRequest(
            "Anime ID must be a positive integer.".to_string(),
        ));
    }

    Ok(())
}

fn validate_status(status: &str) -> Result<(), AppError> {
    if VALID_STATUSES.contains(&status) {
        return Ok(());
    }

    Err(AppError::BadRequest(
        "Status must be one of: watching, completed, dropped, planned.".to_string(),
    ))
}

fn validate_rating(rating: Option<i32>) -> Result<(), AppError> {
    if let Some(value) = rating {
        if !(1..=10).contains(&value) {
            return Err(AppError::BadRequest(
                "Rating must be between 1 and 10.".to_string(),
            ));
        }
    }

    Ok(())
}

fn validate_pagination(limit: Option<i64>, offset: Option<i64>) -> Result<Pagination, AppError> {
    let limit = limit.unwrap_or(DEFAULT_LIST_LIMIT);
    let offset = offset.unwrap_or(0);

    if limit < 1 {
        return Err(AppError::BadRequest(
            "Limit must be at least 1.".to_string(),
        ));
    }

    if limit > MAX_LIST_LIMIT {
        return Err(AppError::BadRequest(format!(
            "Limit must be at most {MAX_LIST_LIMIT}."
        )));
    }

    if offset < 0 {
        return Err(AppError::BadRequest(
            "Offset must be at least 0.".to_string(),
        ));
    }

    Ok(Pagination { limit, offset })
}
