use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::{
    app_state::AppState,
    errors::AppError,
    repositories::recommendation_repository::{self, UserTasteAnime},
    schemas::recommendation::RecommendationResponse,
};

const RELEVANCE_THRESHOLD: f64 = 0.20;
const MAX_ANILIST_QUERIES: usize = 8;
const MAX_RECOMMENDATIONS: usize = 30;
const ANILIST_PER_PAGE: i32 = 15;
const MIN_AVERAGE_SCORE: i32 = 55;
const ANILIST_RETRY_ATTEMPTS: usize = 2;
const ANILIST_RETRY_DELAY_MS: u64 = 900;
const ANILIST_QUERY_DELAY_MS: u64 = 700;
const RECOMMENDATION_CACHE_TTL_MINUTES: i32 = 60;

const RECOMMENDATION_BY_GENRE_QUERY: &str = r#"
query ($genre: String, $page: Int, $perPage: Int, $minScore: Int) {
  Page(page: $page, perPage: $perPage) {
    media(
      genre: $genre,
      type: ANIME,
      sort: [SCORE_DESC, POPULARITY_DESC],
      averageScore_greater: $minScore
    ) {
      id
      title {
        romaji
        english
      }
      genres
      tags {
        name
        category
      }
      coverImage {
        large
      }
      episodes
      description(asHtml: false)
      averageScore
      popularity
    }
  }
}
"#;

const RECOMMENDATION_BY_TAG_QUERY: &str = r#"
query ($tag: String, $page: Int, $perPage: Int, $minScore: Int) {
  Page(page: $page, perPage: $perPage) {
    media(
      tag: $tag,
      type: ANIME,
      sort: [SCORE_DESC, POPULARITY_DESC],
      averageScore_greater: $minScore
    ) {
      id
      title {
        romaji
        english
      }
      genres
      tags {
        name
        category
      }
      coverImage {
        large
      }
      episodes
      description(asHtml: false)
      averageScore
      popularity
    }
  }
}
"#;

#[derive(Debug, Clone)]
struct TasteProfile {
    genres: HashMap<String, f64>,
    tags: HashMap<String, f64>,
    total_weight: f64,
    already_in_list: HashSet<i32>,
}

#[derive(Debug, Clone)]
struct SearchSignal {
    kind: SearchSignalKind,
    name: String,
    relevance: f64,
}

#[derive(Debug, Clone)]
enum SearchSignalKind {
    Genre,
    Tag,
}

#[derive(Debug, Clone)]
struct RankedRecommendation {
    recommendation: RecommendationResponse,
    score: f64,
}

#[derive(Debug, Serialize)]
struct AniListRecommendationRequest<'a> {
    query: &'static str,
    variables: AniListRecommendationVariables<'a>,
}

#[derive(Debug, Serialize)]
struct AniListRecommendationVariables<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    genre: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<&'a str>,

    #[serde(rename = "page")]
    page: i32,

    #[serde(rename = "perPage")]
    per_page: i32,

    #[serde(rename = "minScore")]
    min_score: i32,
}

#[derive(Debug, Deserialize)]
struct AniListRecommendationResponse {
    data: Option<AniListRecommendationData>,
}

#[derive(Debug, Deserialize)]
struct AniListRecommendationData {
    #[serde(rename = "Page")]
    page: Option<AniListPage>,
}

#[derive(Debug, Deserialize)]
struct AniListPage {
    media: Option<Vec<AniListRecommendationMedia>>,
}

#[derive(Debug, Clone, Deserialize)]
struct AniListRecommendationMedia {
    id: i32,
    title: AniListRecommendationTitle,
    genres: Option<Vec<String>>,
    tags: Option<Vec<AniListRecommendationTag>>,

    #[serde(rename = "coverImage")]
    cover_image: Option<AniListRecommendationCoverImage>,

    episodes: Option<i32>,
    description: Option<String>,

    #[serde(rename = "averageScore")]
    average_score: Option<i32>,

    popularity: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
struct AniListRecommendationTitle {
    romaji: Option<String>,
    english: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AniListRecommendationTag {
    name: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AniListRecommendationCoverImage {
    large: Option<String>,
}

pub async fn get_recommendations(
    state: &AppState,
    user_id: i32,
    refresh: bool,
) -> Result<Vec<RecommendationResponse>, AppError> {
    let rows = recommendation_repository::list_user_taste_anime(&state.db, user_id).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let profile = build_taste_profile(&rows);

    if profile.total_weight <= 0.0 {
        return Ok(Vec::new());
    }

    if !refresh {
        if let Some(cached_recommendations) =
            read_valid_cached_recommendations(state, user_id, &profile).await?
        {
            return Ok(cached_recommendations);
        }
    }

    let signals = select_search_signals(&profile);

    if signals.is_empty() {
        return Ok(Vec::new());
    }

    let mut seen_ids = HashSet::new();
    let mut ranked = Vec::new();
    let mut successful_queries = 0usize;
    let mut failed_queries = 0usize;

    for signal in signals.into_iter().take(MAX_ANILIST_QUERIES) {
        let candidates =
            match fetch_candidates(&state.http_client, &state.config.anilist_url, &signal).await {
                Ok(candidates) => {
                    successful_queries += 1;
                    candidates
                }
                Err(AppError::TooManyRequests(detail)) => {
                    if let Some(cached_recommendations) =
                        read_latest_cached_recommendations(state, user_id, &profile).await?
                    {
                        return Ok(cached_recommendations);
                    }

                    return Err(AppError::TooManyRequests(detail));
                }
                Err(error) => {
                    failed_queries += 1;
                    tracing::warn!(
                    "Skipping recommendation signal after AniList failure. signal={:?}, error={:?}",
                    signal,
                    error
                );
                    sleep(Duration::from_millis(ANILIST_QUERY_DELAY_MS)).await;
                    continue;
                }
            };

        for candidate in candidates {
            if profile.already_in_list.contains(&candidate.id) || seen_ids.contains(&candidate.id) {
                continue;
            }

            let Some(recommendation) = parse_candidate(&candidate) else {
                continue;
            };

            let score = rank_candidate(&candidate, &profile, signal.relevance);

            seen_ids.insert(candidate.id);
            ranked.push(RankedRecommendation {
                recommendation,
                score,
            });
        }

        sleep(Duration::from_millis(ANILIST_QUERY_DELAY_MS)).await;
    }

    if successful_queries == 0 && failed_queries > 0 {
        if let Some(cached_recommendations) =
            read_latest_cached_recommendations(state, user_id, &profile).await?
        {
            return Ok(cached_recommendations);
        }

        return Err(AppError::ServiceUnavailable(
            "AniList is currently unavailable.".to_string(),
        ));
    }

    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
    });

    let recommendations = ranked
        .into_iter()
        .take(MAX_RECOMMENDATIONS)
        .map(|ranked| ranked.recommendation)
        .collect::<Vec<_>>();

    if !recommendations.is_empty() {
        write_recommendation_cache(state, user_id, &recommendations).await;
    }

    Ok(recommendations)
}

async fn read_valid_cached_recommendations(
    state: &AppState,
    user_id: i32,
    profile: &TasteProfile,
) -> Result<Option<Vec<RecommendationResponse>>, AppError> {
    let Some(data) = recommendation_repository::find_valid_recommendation_cache(
        &state.db,
        user_id,
        RECOMMENDATION_CACHE_TTL_MINUTES,
    )
    .await?
    else {
        return Ok(None);
    };

    Ok(parse_cached_recommendations(&data, profile))
}

async fn read_latest_cached_recommendations(
    state: &AppState,
    user_id: i32,
    profile: &TasteProfile,
) -> Result<Option<Vec<RecommendationResponse>>, AppError> {
    let Some(data) =
        recommendation_repository::find_latest_recommendation_cache(&state.db, user_id).await?
    else {
        return Ok(None);
    };

    Ok(parse_cached_recommendations(&data, profile))
}

fn parse_cached_recommendations(
    data: &str,
    profile: &TasteProfile,
) -> Option<Vec<RecommendationResponse>> {
    let recommendations = match serde_json::from_str::<Vec<RecommendationResponse>>(data) {
        Ok(recommendations) => recommendations,
        Err(error) => {
            tracing::warn!("Invalid recommendation cache JSON. error={error}");
            return None;
        }
    };

    Some(
        recommendations
            .into_iter()
            .filter(|recommendation| !profile.already_in_list.contains(&recommendation.anilist_id))
            .collect(),
    )
}

async fn write_recommendation_cache(
    state: &AppState,
    user_id: i32,
    recommendations: &[RecommendationResponse],
) {
    let data = match serde_json::to_string(recommendations) {
        Ok(data) => data,
        Err(error) => {
            tracing::warn!("Could not serialize recommendations cache. error={error}");
            return;
        }
    };

    if let Err(error) =
        recommendation_repository::replace_recommendation_cache(&state.db, user_id, &data).await
    {
        tracing::warn!("Could not write recommendation cache. error={error}");
    }
}

fn build_taste_profile(rows: &[UserTasteAnime]) -> TasteProfile {
    let mut genres = HashMap::new();
    let mut tags = HashMap::new();
    let mut already_in_list = HashSet::new();
    let mut total_weight = 0.0;

    for row in rows {
        already_in_list.insert(row.anilist_id);

        let weight = anime_weight(row);
        total_weight += weight;

        for genre in split_csv(row.genres.as_deref()) {
            *genres.entry(genre).or_insert(0.0) += weight;
        }

        for tag in split_csv(row.tags.as_deref()) {
            if is_ignored_tag(&tag) {
                continue;
            }

            *tags.entry(tag).or_insert(0.0) += weight;
        }
    }

    TasteProfile {
        genres,
        tags,
        total_weight,
        already_in_list,
    }
}

fn anime_weight(row: &UserTasteAnime) -> f64 {
    let status_weight: f64 = match row.status.as_str() {
        "completed" => 1.00,
        "watching" => 0.90,
        "planned" => 0.35,
        "dropped" => 0.10,
        _ => 0.25,
    };

    let rating_bonus: f64 = match row.rating {
        Some(9..=10) => 0.60,
        Some(7..=8) => 0.30,
        Some(5..=6) | None => 0.00,
        Some(1..=4) => -0.30,
        Some(_) => 0.00,
    };

    let favorite_bonus: f64 = if row.is_favorite { 0.75 } else { 0.00 };

    (status_weight + rating_bonus + favorite_bonus).max(0.05_f64)
}

fn select_search_signals(profile: &TasteProfile) -> Vec<SearchSignal> {
    let mut signals = Vec::new();

    for (genre, score) in top_relevant_items(&profile.genres, profile.total_weight, 5) {
        signals.push(SearchSignal {
            kind: SearchSignalKind::Genre,
            name: genre,
            relevance: score,
        });
    }

    for (tag, score) in top_relevant_items(&profile.tags, profile.total_weight, 8) {
        signals.push(SearchSignal {
            kind: SearchSignalKind::Tag,
            name: tag,
            relevance: score,
        });
    }

    signals.sort_by(|left, right| {
        right
            .relevance
            .partial_cmp(&left.relevance)
            .unwrap_or(Ordering::Equal)
    });

    signals
}

fn top_relevant_items(
    items: &HashMap<String, f64>,
    total_weight: f64,
    limit: usize,
) -> Vec<(String, f64)> {
    let mut scored = items
        .iter()
        .filter_map(|(name, score)| {
            let relevance = score / total_weight;

            if relevance >= RELEVANCE_THRESHOLD {
                Some((name.clone(), relevance))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(Ordering::Equal));
    scored.truncate(limit);
    scored
}

async fn fetch_candidates(
    client: &Client,
    anilist_url: &str,
    signal: &SearchSignal,
) -> Result<Vec<AniListRecommendationMedia>, AppError> {
    let mut last_error = None;

    for attempt in 1..=ANILIST_RETRY_ATTEMPTS {
        match fetch_candidates_once(client, anilist_url, signal).await {
            Ok(candidates) => return Ok(candidates),
            Err(AppError::TooManyRequests(detail)) => {
                return Err(AppError::TooManyRequests(detail));
            }
            Err(error) => {
                last_error = Some(error);

                if attempt < ANILIST_RETRY_ATTEMPTS {
                    tracing::warn!(
                        "Retrying AniList recommendation request. attempt={attempt}, signal={:?}",
                        signal
                    );
                    sleep(Duration::from_millis(ANILIST_RETRY_DELAY_MS)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
    }))
}

async fn fetch_candidates_once(
    client: &Client,
    anilist_url: &str,
    signal: &SearchSignal,
) -> Result<Vec<AniListRecommendationMedia>, AppError> {
    let (query, genre, tag) = match signal.kind {
        SearchSignalKind::Genre => (
            RECOMMENDATION_BY_GENRE_QUERY,
            Some(signal.name.as_str()),
            None,
        ),
        SearchSignalKind::Tag => (
            RECOMMENDATION_BY_TAG_QUERY,
            None,
            Some(signal.name.as_str()),
        ),
    };

    let response = client
        .post(anilist_url)
        .json(&AniListRecommendationRequest {
            query,
            variables: AniListRecommendationVariables {
                genre,
                tag,
                page: 1,
                per_page: ANILIST_PER_PAGE,
                min_score: MIN_AVERAGE_SCORE,
            },
        })
        .send()
        .await
        .map_err(|error| {
            tracing::error!(
                "Could not reach AniList recommendations endpoint. signal={:?}, error={error}",
                signal
            );
            AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
        })?;

    let status = response.status();
    let response_text = response.text().await.map_err(|error| {
        tracing::error!(
            "Could not read AniList recommendations response body. signal={:?}, error={error}",
            signal
        );
        AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
    })?;

    if status == StatusCode::TOO_MANY_REQUESTS {
        return Err(AppError::TooManyRequests(
            "AniList rate limit exceeded. Try again later.".to_string(),
        ));
    }

    if !status.is_success() {
        tracing::warn!(
            "AniList recommendations returned status {status}. signal={:?}, body={}",
            signal,
            response_text
        );

        return Err(AppError::ServiceUnavailable(
            "AniList is currently unavailable.".to_string(),
        ));
    }

    let body = serde_json::from_str::<AniListRecommendationResponse>(&response_text).map_err(
        |error| {
            tracing::error!(
                "Invalid AniList recommendations response body. signal={:?}, error={error}, body={}",
                signal,
                response_text
            );
            AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
        },
    )?;

    Ok(body
        .data
        .and_then(|data| data.page)
        .and_then(|page| page.media)
        .unwrap_or_default())
}

fn parse_candidate(media: &AniListRecommendationMedia) -> Option<RecommendationResponse> {
    let title_romaji = media
        .title
        .romaji
        .clone()
        .or_else(|| media.title.english.clone())?;

    let genres = media.genres.clone().unwrap_or_default();

    if genres.is_empty()
        || media
            .cover_image
            .as_ref()
            .and_then(|cover| cover.large.as_ref())
            .is_none()
    {
        return None;
    }

    let tags = media
        .tags
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tag| {
            if tag.category.as_deref() == Some("Demographic") {
                return None;
            }

            tag.name
        })
        .filter(|tag| !is_ignored_tag(tag))
        .collect::<Vec<_>>();

    Some(RecommendationResponse {
        anilist_id: media.id,
        title_romaji,
        title_english: media.title.english.clone(),
        genres,
        tags,
        cover_image_url: media
            .cover_image
            .as_ref()
            .and_then(|cover| cover.large.clone()),
        episodes: media.episodes,
        description: media.description.clone(),
    })
}

fn rank_candidate(
    media: &AniListRecommendationMedia,
    profile: &TasteProfile,
    signal_relevance: f64,
) -> f64 {
    let candidate_genres = media.genres.clone().unwrap_or_default();
    let candidate_tags = media
        .tags
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tag| tag.name)
        .filter(|tag| !is_ignored_tag(tag))
        .collect::<Vec<_>>();

    let genre_score = candidate_genres
        .iter()
        .filter_map(|genre| profile.genres.get(genre))
        .sum::<f64>();

    let tag_score = candidate_tags
        .iter()
        .filter_map(|tag| profile.tags.get(tag))
        .sum::<f64>();

    let average_score_bonus = media.average_score.unwrap_or(0) as f64 / 100.0;
    let popularity_bonus = media.popularity.unwrap_or(0).min(500_000) as f64 / 500_000.0;

    signal_relevance * 2.0
        + genre_score * 2.0
        + tag_score * 1.2
        + average_score_bonus
        + popularity_bonus * 0.25
}

fn split_csv(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn is_ignored_tag(tag: &str) -> bool {
    matches!(
        tag,
        "Male Protagonist"
            | "Female Protagonist"
            | "Primarily Male Cast"
            | "Primarily Female Cast"
            | "Ensemble Cast"
            | "Coming of Age"
            | "School"
            | "Urban"
            | "Large Breasts"
            | "Episodic"
            | "Tragedy"
    )
}
