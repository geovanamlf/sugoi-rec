use chrono::{NaiveDateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

const ANIME_QUERY: &str = r#"
query ($id: Int, $search: String) {
  Media(id: $id, search: $search, type: ANIME) {
    id
    title {
      romaji
      english
      native
    }
    episodes
    coverImage {
      large
    }
    description(asHtml: false)
    genres
    tags {
      name
      category
    }
  }
}
"#;

#[derive(Debug, Clone)]
pub struct ParsedAnimeData {
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

#[derive(Debug, Serialize)]
struct AniListRequest {
    query: &'static str,
    variables: AniListVariables,
}

#[derive(Debug, Serialize)]
struct AniListVariables {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    search: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AniListResponse {
    data: Option<AniListData>,
    errors: Option<Vec<AniListGraphQlError>>,
}

#[derive(Debug, Deserialize)]
struct AniListGraphQlError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct AniListData {
    #[serde(rename = "Media")]
    media: Option<AniListMedia>,
}

#[derive(Debug, Deserialize)]
struct AniListMedia {
    id: i32,
    title: AniListTitle,
    episodes: Option<i32>,

    #[serde(rename = "coverImage")]
    cover_image: Option<AniListCoverImage>,

    description: Option<String>,
    genres: Option<Vec<String>>,
    tags: Option<Vec<AniListTag>>,
}

#[derive(Debug, Deserialize)]
struct AniListTitle {
    romaji: Option<String>,
    english: Option<String>,
    native: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AniListCoverImage {
    large: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AniListTag {
    name: Option<String>,
    category: Option<String>,
}

pub async fn fetch_anime_by_id(
    client: &Client,
    anilist_url: &str,
    anilist_id: i32,
) -> Result<ParsedAnimeData, AppError> {
    request(
        client,
        anilist_url,
        AniListVariables {
            id: Some(anilist_id),
            search: None,
        },
    )
    .await
}

pub async fn fetch_anime_by_name(
    client: &Client,
    anilist_url: &str,
    search: &str,
) -> Result<ParsedAnimeData, AppError> {
    request(
        client,
        anilist_url,
        AniListVariables {
            id: None,
            search: Some(search.to_string()),
        },
    )
    .await
}

async fn request(
    client: &Client,
    anilist_url: &str,
    variables: AniListVariables,
) -> Result<ParsedAnimeData, AppError> {
    let response = client
        .post(anilist_url)
        .json(&AniListRequest {
            query: ANIME_QUERY,
            variables,
        })
        .send()
        .await
        .map_err(|error| {
            tracing::error!("Could not reach AniList: {error}");
            AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
        })?;

    let status = response.status();

    if status == StatusCode::TOO_MANY_REQUESTS {
        return Err(AppError::TooManyRequests(
            "AniList rate limit exceeded. Try again later.".to_string(),
        ));
    }

    let response_text = response.text().await.map_err(|error| {
        tracing::error!("Could not read AniList response body: {error}");
        AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
    })?;

    if !status.is_success() {
        tracing::warn!("AniList returned status {status}. Body: {response_text}");

        if status == StatusCode::NOT_FOUND
            || status == StatusCode::BAD_REQUEST
            || response_text.to_lowercase().contains("not found")
        {
            return Err(AppError::NotFound("Anime not found.".to_string()));
        }

        return Err(AppError::ServiceUnavailable(
            "AniList is currently unavailable.".to_string(),
        ));
    }

    let body = serde_json::from_str::<AniListResponse>(&response_text).map_err(|error| {
        tracing::error!("Invalid AniList response body: {error}. Body: {response_text}");
        AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
    })?;

    if let Some(errors) = &body.errors {
        let messages = errors
            .iter()
            .map(|error| error.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ");

        tracing::warn!("AniList GraphQL errors: {messages}");

        if messages.to_lowercase().contains("not found") {
            return Err(AppError::NotFound("Anime not found.".to_string()));
        }
    }

    let media = body
        .data
        .and_then(|data| data.media)
        .ok_or_else(|| AppError::NotFound("Anime not found.".to_string()))?;

    Ok(parse_media(media))
}

fn parse_media(media: AniListMedia) -> ParsedAnimeData {
    let title_romaji = media.title.romaji.clone().unwrap_or_else(|| {
        media.title.english.clone().unwrap_or_else(|| {
            media
                .title
                .native
                .clone()
                .unwrap_or_else(|| format!("Anime #{}", media.id))
        })
    });

    let genres = join_or_none(media.genres.unwrap_or_default());

    let mut tags = Vec::new();
    let mut demographic = None;

    for tag in media.tags.unwrap_or_default() {
        let is_demographic = tag.category.as_deref() == Some("Demographic");

        if let Some(name) = tag.name {
            if is_demographic && demographic.is_none() {
                demographic = Some(name);
            } else if !is_demographic {
                tags.push(name);
            }
        }
    }

    ParsedAnimeData {
        anilist_id: media.id,
        title_romaji,
        title_english: media.title.english,
        title_native: media.title.native,
        episode_count: media.episodes,
        cover_image_url: media.cover_image.and_then(|cover| cover.large),
        description: media.description,
        genres,
        tags: join_or_none(tags),
        demographic,
        cached_at: Utc::now().naive_utc(),
    }
}

fn join_or_none(items: Vec<String>) -> Option<String> {
    if items.is_empty() {
        None
    } else {
        Some(items.join(","))
    }
}
