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
const MAX_CONTINUATIONS: usize = 20;
const MAX_CONTINUATION_SOURCE_ANIME: usize = 8;
const ANILIST_PER_PAGE: i32 = 50;
const ANILIST_MAX_PAGES_PER_SIGNAL: i32 = 2;
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
      relations {
        edges {
          relationType
          node {
            id
          }
        }
      }
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
      relations {
        edges {
          relationType
          node {
            id
          }
        }
      }
    }
  }
}
"#;

const CONTINUATIONS_BY_ANIME_QUERY: &str = r#"
query ($id: Int) {
  Media(id: $id, type: ANIME) {
    id
    relations {
      edges {
        relationType
        node {
          id
          type
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
  }
}
"#;

#[derive(Debug, Clone)]
struct TasteProfile {
    genres: HashMap<String, f64>,
    tags: HashMap<String, f64>,
    total_weight: f64,
    already_in_list: HashSet<i32>,
    title_roots: HashSet<String>,
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
struct AniListContinuationRequest {
    query: &'static str,
    variables: AniListContinuationVariables,
}

#[derive(Debug, Serialize)]
struct AniListContinuationVariables {
    id: i32,
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
struct AniListContinuationResponse {
    data: Option<AniListContinuationData>,
}

#[derive(Debug, Deserialize)]
struct AniListContinuationData {
    #[serde(rename = "Media")]
    media: Option<AniListContinuationMedia>,
}

#[derive(Debug, Deserialize)]
struct AniListContinuationMedia {
    relations: Option<AniListContinuationRelations>,
}

#[derive(Debug, Deserialize)]
struct AniListContinuationRelations {
    edges: Option<Vec<AniListContinuationRelationEdge>>,
}

#[derive(Debug, Deserialize)]
struct AniListContinuationRelationEdge {
    #[serde(rename = "relationType")]
    relation_type: Option<String>,

    node: Option<AniListContinuationNode>,
}

#[derive(Debug, Clone, Deserialize)]
struct AniListContinuationNode {
    id: i32,

    #[serde(rename = "type")]
    media_type: Option<String>,

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

    relations: Option<AniListRecommendationRelations>,
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

#[derive(Debug, Clone, Deserialize)]
struct AniListRecommendationRelations {
    edges: Option<Vec<AniListRecommendationRelationEdge>>,
}

#[derive(Debug, Clone, Deserialize)]
struct AniListRecommendationRelationEdge {
    #[serde(rename = "relationType")]
    relation_type: Option<String>,

    node: Option<AniListRecommendationRelationNode>,
}

#[derive(Debug, Clone, Deserialize)]
struct AniListRecommendationRelationNode {
    id: i32,
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
            if should_skip_candidate(&candidate, &profile, &seen_ids) {
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

pub async fn get_continuations(
    state: &AppState,
    user_id: i32,
) -> Result<Vec<RecommendationResponse>, AppError> {
    let rows = recommendation_repository::list_user_taste_anime(&state.db, user_id).await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let profile = build_taste_profile(&rows);
    let mut seen_ids = HashSet::new();
    let mut ranked = Vec::new();

    let mut source_rows = rows
        .iter()
        .filter(|row| should_fetch_continuations_for_status(&row.status))
        .collect::<Vec<_>>();

    source_rows.sort_by(|left, right| {
        anime_weight(right)
            .partial_cmp(&anime_weight(left))
            .unwrap_or(Ordering::Equal)
    });

    for row in source_rows.into_iter().take(MAX_CONTINUATION_SOURCE_ANIME) {
        let candidates = match fetch_continuation_candidates(
            &state.http_client,
            &state.config.anilist_url,
            row.anilist_id,
        )
        .await
        {
            Ok(candidates) => candidates,
            Err(AppError::TooManyRequests(detail)) => {
                if !ranked.is_empty() {
                    tracing::warn!(
                        anime_id = row.anilist_id,
                        "AniList rate limit while fetching continuations; returning partial results"
                    );
                    break;
                }

                return Err(AppError::TooManyRequests(detail));
            }
            Err(error) => {
                tracing::warn!(
                    anime_id = row.anilist_id,
                    "Skipping continuation lookup after AniList failure. error={:?}",
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

            let Some(recommendation) = parse_continuation_candidate(&candidate) else {
                continue;
            };

            let score = rank_continuation_candidate(&candidate);

            seen_ids.insert(candidate.id);
            ranked.push(RankedRecommendation {
                recommendation,
                score,
            });
        }

        sleep(Duration::from_millis(ANILIST_QUERY_DELAY_MS)).await;
    }

    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
    });

    Ok(ranked
        .into_iter()
        .take(MAX_CONTINUATIONS)
        .map(|ranked| ranked.recommendation)
        .collect())
}

fn should_fetch_continuations_for_status(status: &str) -> bool {
    matches!(status, "completed" | "watching")
}

async fn fetch_continuation_candidates(
    client: &Client,
    anilist_url: &str,
    anilist_id: i32,
) -> Result<Vec<AniListContinuationNode>, AppError> {
    let mut last_error = None;

    for attempt in 1..=ANILIST_RETRY_ATTEMPTS {
        match fetch_continuation_candidates_once(client, anilist_url, anilist_id).await {
            Ok(candidates) => return Ok(candidates),
            Err(AppError::TooManyRequests(detail)) => {
                return Err(AppError::TooManyRequests(detail));
            }
            Err(error) => {
                last_error = Some(error);

                if attempt < ANILIST_RETRY_ATTEMPTS {
                    tracing::warn!(
                        "Retrying AniList continuation request. attempt={attempt}, anime_id={anilist_id}"
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

async fn fetch_continuation_candidates_once(
    client: &Client,
    anilist_url: &str,
    anilist_id: i32,
) -> Result<Vec<AniListContinuationNode>, AppError> {
    let response = client
        .post(anilist_url)
        .json(&AniListContinuationRequest {
            query: CONTINUATIONS_BY_ANIME_QUERY,
            variables: AniListContinuationVariables { id: anilist_id },
        })
        .send()
        .await
        .map_err(|error| {
            tracing::error!(
                "Could not reach AniList continuations endpoint. anime_id={anilist_id}, error={error}"
            );
            AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
        })?;

    let status = response.status();
    let response_text = response.text().await.map_err(|error| {
        tracing::error!(
            "Could not read AniList continuations response body. anime_id={anilist_id}, error={error}"
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
            "AniList continuations returned status {status}. anime_id={anilist_id}, body={}",
            response_text
        );

        return Err(AppError::ServiceUnavailable(
            "AniList is currently unavailable.".to_string(),
        ));
    }

    let body = serde_json::from_str::<AniListContinuationResponse>(&response_text).map_err(
        |error| {
            tracing::error!(
                "Invalid AniList continuations response body. anime_id={anilist_id}, error={error}, body={}",
                response_text
            );
            AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
        },
    )?;

    let candidates = body
        .data
        .and_then(|data| data.media)
        .and_then(|media| media.relations)
        .and_then(|relations| relations.edges)
        .unwrap_or_default()
        .into_iter()
        .filter(|edge| edge.relation_type.as_deref() == Some("SEQUEL"))
        .filter_map(|edge| edge.node)
        .filter(|node| node.media_type.as_deref() == Some("ANIME"))
        .collect::<Vec<_>>();

    Ok(candidates)
}

fn parse_continuation_candidate(media: &AniListContinuationNode) -> Option<RecommendationResponse> {
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

fn rank_continuation_candidate(media: &AniListContinuationNode) -> f64 {
    let average_score_bonus = media.average_score.unwrap_or(0) as f64 / 100.0;
    let popularity_bonus = media.popularity.unwrap_or(0).min(500_000) as f64 / 500_000.0;

    average_score_bonus + popularity_bonus * 0.25
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

    let had_cached_recommendations = !recommendations.is_empty();

    let filtered_recommendations = recommendations
        .into_iter()
        .filter(|recommendation| !should_skip_cached_recommendation(recommendation, profile))
        .collect::<Vec<_>>();

    if had_cached_recommendations && filtered_recommendations.is_empty() {
        return None;
    }

    Some(filtered_recommendations)
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
    let mut title_roots = HashSet::new();
    let mut total_weight = 0.0;

    for row in rows {
        already_in_list.insert(row.anilist_id);

        if let Some(title_root) = title_family_root(&row.title_romaji) {
            title_roots.insert(title_root);
        }

        if let Some(title_english) = row.title_english.as_deref() {
            if let Some(title_root) = title_family_root(title_english) {
                title_roots.insert(title_root);
            }
        }

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
        title_roots,
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
    let mut all_candidates = Vec::new();
    let mut last_error = None;

    for page in 1..=ANILIST_MAX_PAGES_PER_SIGNAL {
        let mut page_loaded = false;

        for attempt in 1..=ANILIST_RETRY_ATTEMPTS {
            match fetch_candidates_once(client, anilist_url, signal, page).await {
                Ok(candidates) => {
                    page_loaded = true;

                    if candidates.is_empty() {
                        return Ok(all_candidates);
                    }

                    all_candidates.extend(candidates);
                    break;
                }
                Err(AppError::TooManyRequests(detail)) => {
                    if !all_candidates.is_empty() {
                        tracing::warn!(
                            "AniList rate limit while fetching extra recommendation pages; returning partial candidates. signal={:?}, page={page}",
                            signal
                        );
                        return Ok(all_candidates);
                    }

                    return Err(AppError::TooManyRequests(detail));
                }
                Err(error) => {
                    last_error = Some(error);

                    if attempt < ANILIST_RETRY_ATTEMPTS {
                        tracing::warn!(
                            "Retrying AniList recommendation request. attempt={attempt}, signal={:?}, page={page}",
                            signal
                        );
                        sleep(Duration::from_millis(ANILIST_RETRY_DELAY_MS)).await;
                    }
                }
            }
        }

        if !page_loaded {
            break;
        }

        if page < ANILIST_MAX_PAGES_PER_SIGNAL {
            sleep(Duration::from_millis(ANILIST_QUERY_DELAY_MS)).await;
        }
    }

    if !all_candidates.is_empty() {
        return Ok(all_candidates);
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::ServiceUnavailable("AniList is currently unavailable.".to_string())
    }))
}

async fn fetch_candidates_once(
    client: &Client,
    anilist_url: &str,
    signal: &SearchSignal,
    page: i32,
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
                page,
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

fn should_skip_candidate(
    candidate: &AniListRecommendationMedia,
    profile: &TasteProfile,
    seen_ids: &HashSet<i32>,
) -> bool {
    seen_ids.contains(&candidate.id)
        || profile.already_in_list.contains(&candidate.id)
        || is_dependent_work_by_anilist_relation(candidate)
        || is_related_to_user_list_by_anilist_relation(candidate, profile)
        || is_related_to_user_list_by_title_family(candidate, profile)
        || is_obvious_dependent_work_by_title(candidate)
}

fn should_skip_cached_recommendation(
    recommendation: &RecommendationResponse,
    profile: &TasteProfile,
) -> bool {
    if profile.already_in_list.contains(&recommendation.anilist_id) {
        return true;
    }

    recommendation_title_roots(recommendation)
        .iter()
        .any(|candidate_root| {
            profile
                .title_roots
                .iter()
                .any(|user_root| title_roots_look_related(candidate_root, user_root))
        })
}

fn is_dependent_work_by_anilist_relation(candidate: &AniListRecommendationMedia) -> bool {
    let Some(edges) = candidate
        .relations
        .as_ref()
        .and_then(|relations| relations.edges.as_ref())
    else {
        return false;
    };

    edges.iter().any(|edge| {
        edge.relation_type
            .as_deref()
            .is_some_and(is_global_dependency_relation_type)
    })
}

fn is_related_to_user_list_by_anilist_relation(
    candidate: &AniListRecommendationMedia,
    profile: &TasteProfile,
) -> bool {
    let Some(edges) = candidate
        .relations
        .as_ref()
        .and_then(|relations| relations.edges.as_ref())
    else {
        return false;
    };

    edges.iter().any(|edge| {
        let Some(relation_type) = edge.relation_type.as_deref() else {
            return false;
        };

        let Some(node) = edge.node.as_ref() else {
            return false;
        };

        is_blocked_relation_type(relation_type) && profile.already_in_list.contains(&node.id)
    })
}

fn is_related_to_user_list_by_title_family(
    candidate: &AniListRecommendationMedia,
    profile: &TasteProfile,
) -> bool {
    candidate_title_roots(candidate)
        .iter()
        .any(|candidate_root| {
            profile
                .title_roots
                .iter()
                .any(|user_root| title_roots_look_related(candidate_root, user_root))
        })
}

fn is_obvious_dependent_work_by_title(candidate: &AniListRecommendationMedia) -> bool {
    candidate_raw_titles(candidate)
        .iter()
        .any(|title| title_has_dependent_work_marker(title))
}

fn candidate_raw_titles(candidate: &AniListRecommendationMedia) -> Vec<String> {
    let mut titles = Vec::new();

    if let Some(title) = candidate.title.romaji.as_deref() {
        titles.push(title.to_string());
    }

    if let Some(title) = candidate.title.english.as_deref() {
        titles.push(title.to_string());
    }

    titles
}

fn title_has_dependent_work_marker(title: &str) -> bool {
    let normalized = title
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>();

    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    if tokens.iter().any(|token| {
        matches!(
            *token,
            "season"
                | "movie"
                | "movies"
                | "ova"
                | "ovas"
                | "ona"
                | "onas"
                | "special"
                | "specials"
                | "recap"
                | "summary"
                | "summaries"
                | "final"
                | "part"
                | "cour"
                | "kanketsu"
                | "kouhan"
                | "second"
                | "third"
                | "fourth"
                | "fan"
                | "letter"
        )
    }) {
        return true;
    }

    tokens.last().is_some_and(|last_token| {
        last_token
            .chars()
            .all(|character| character.is_ascii_digit())
    })
}

fn is_global_dependency_relation_type(relation_type: &str) -> bool {
    matches!(relation_type, "PREQUEL" | "PARENT" | "SUMMARY")
}

fn is_blocked_relation_type(relation_type: &str) -> bool {
    matches!(
        relation_type,
        "SEQUEL"
            | "PREQUEL"
            | "SIDE_STORY"
            | "PARENT"
            | "SUMMARY"
            | "SPIN_OFF"
            | "ALTERNATIVE"
            | "COMPILATION"
    )
}

fn candidate_title_roots(candidate: &AniListRecommendationMedia) -> Vec<String> {
    let mut roots = Vec::new();

    if let Some(root) = candidate
        .title
        .romaji
        .as_deref()
        .and_then(title_family_root)
    {
        roots.push(root);
    }

    if let Some(root) = candidate
        .title
        .english
        .as_deref()
        .and_then(title_family_root)
    {
        roots.push(root);
    }

    roots
}

fn recommendation_title_roots(recommendation: &RecommendationResponse) -> Vec<String> {
    let mut roots = Vec::new();

    if let Some(root) = title_family_root(&recommendation.title_romaji) {
        roots.push(root);
    }

    if let Some(root) = recommendation
        .title_english
        .as_deref()
        .and_then(title_family_root)
    {
        roots.push(root);
    }

    roots
}

fn title_roots_look_related(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }

    if !title_root_is_specific_enough(left) || !title_root_is_specific_enough(right) {
        return false;
    }

    left.starts_with(right) || right.starts_with(left)
}

fn title_root_is_specific_enough(root: &str) -> bool {
    root.len() >= 10 && root.split_whitespace().count() >= 2
}

fn title_family_root(title: &str) -> Option<String> {
    let normalized = title
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>();

    let tokens = normalized
        .split_whitespace()
        .filter(|token| !token.chars().any(|character| character.is_ascii_digit()))
        .filter(|token| !is_title_family_noise_token(token))
        .collect::<Vec<_>>();

    if tokens.is_empty() {
        return None;
    }

    Some(tokens.join(" "))
}

fn is_title_family_noise_token(token: &str) -> bool {
    matches!(
        token,
        "season"
            | "seasons"
            | "movie"
            | "movies"
            | "film"
            | "films"
            | "ova"
            | "ovas"
            | "ona"
            | "onas"
            | "special"
            | "specials"
            | "recap"
            | "recaps"
            | "summary"
            | "summaries"
            | "final"
            | "part"
            | "cour"
            | "episode"
            | "episodes"
            | "edition"
            | "remake"
            | "kanketsu"
            | "hen"
            | "kai"
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn profile(already_ids: &[i32], title_roots: &[&str]) -> TasteProfile {
        TasteProfile {
            genres: HashMap::new(),
            tags: HashMap::new(),
            total_weight: 1.0,
            already_in_list: already_ids.iter().copied().collect(),
            title_roots: title_roots.iter().map(|title| title.to_string()).collect(),
        }
    }

    fn candidate(
        id: i32,
        romaji: &str,
        english: Option<&str>,
        relations: Vec<(&str, i32)>,
    ) -> AniListRecommendationMedia {
        AniListRecommendationMedia {
            id,
            title: AniListRecommendationTitle {
                romaji: Some(romaji.to_string()),
                english: english.map(ToString::to_string),
            },
            genres: Some(vec!["Action".to_string()]),
            tags: None,
            cover_image: None,
            episodes: None,
            description: None,
            average_score: None,
            popularity: None,
            relations: if relations.is_empty() {
                None
            } else {
                Some(AniListRecommendationRelations {
                    edges: Some(
                        relations
                            .into_iter()
                            .map(
                                |(relation_type, related_id)| AniListRecommendationRelationEdge {
                                    relation_type: Some(relation_type.to_string()),
                                    node: Some(AniListRecommendationRelationNode {
                                        id: related_id,
                                    }),
                                },
                            )
                            .collect(),
                    ),
                })
            },
        }
    }

    #[test]
    fn title_family_root_removes_common_dependent_work_noise() {
        assert_eq!(
            title_family_root("Attack on Titan Season 2"),
            Some("attack on titan".to_string())
        );

        assert_eq!(
            title_family_root("Violet Evergarden Movie"),
            Some("violet evergarden".to_string())
        );

        assert_eq!(
            title_family_root("3-gatsu no Lion 2nd Season"),
            Some("gatsu no lion".to_string())
        );
    }

    #[test]
    fn build_taste_profile_indexes_romaji_and_english_title_roots() {
        let rows = vec![UserTasteAnime {
            status: "completed".to_string(),
            rating: Some(10),
            is_favorite: true,
            anilist_id: 16498,
            title_romaji: "Shingeki no Kyojin".to_string(),
            title_english: Some("Attack on Titan".to_string()),
            genres: Some("Action,Drama".to_string()),
            tags: None,
        }];

        let profile = build_taste_profile(&rows);

        assert!(profile.already_in_list.contains(&16498));
        assert!(profile.title_roots.contains("shingeki no kyojin"));
        assert!(profile.title_roots.contains("attack on titan"));
    }

    #[test]
    fn skips_exact_anime_already_in_user_list() {
        let profile = profile(&[16498], &["attack on titan"]);
        let seen_ids = HashSet::new();
        let candidate = candidate(16498, "Shingeki no Kyojin", Some("Attack on Titan"), vec![]);

        assert!(should_skip_candidate(&candidate, &profile, &seen_ids));
    }

    #[test]
    fn skips_candidate_with_same_title_family_as_user_anime() {
        let profile = profile(&[16498], &["attack on titan"]);
        let seen_ids = HashSet::new();
        let candidate = candidate(
            25777,
            "Shingeki no Kyojin 2",
            Some("Attack on Titan Season 2"),
            vec![],
        );

        assert!(should_skip_candidate(&candidate, &profile, &seen_ids));
    }

    #[test]
    fn skips_candidate_related_to_user_anime_by_relation() {
        let profile = profile(&[16498], &["attack on titan"]);
        let seen_ids = HashSet::new();
        let candidate = candidate(
            25777,
            "Shingeki no Kyojin 2",
            Some("Attack on Titan Season 2"),
            vec![("PREQUEL", 16498)],
        );

        assert!(should_skip_candidate(&candidate, &profile, &seen_ids));
    }

    #[test]
    fn skips_globally_dependent_work_with_prequel_relation() {
        let profile = profile(&[], &[]);
        let seen_ids = HashSet::new();
        let candidate = candidate(
            1001,
            "Made in Abyss: Fukaki Tamashii no Reimei",
            Some("Made in Abyss: Dawn of the Deep Soul"),
            vec![("PREQUEL", 999)],
        );

        assert!(should_skip_candidate(&candidate, &profile, &seen_ids));
    }

    #[test]
    fn does_not_skip_main_work_just_because_it_has_a_sequel() {
        let profile = profile(&[], &[]);
        let seen_ids = HashSet::new();
        let candidate = candidate(
            1002,
            "Dungeon Meshi",
            Some("Delicious in Dungeon"),
            vec![("SEQUEL", 2002)],
        );

        assert!(!should_skip_candidate(&candidate, &profile, &seen_ids));
    }

    #[test]
    fn skips_obvious_dependent_work_title_markers() {
        let profile = profile(&[], &[]);
        let seen_ids = HashSet::new();

        let season_two = candidate(
            2001,
            "Tian Guan Ci Fu 2",
            Some("Heaven Official's Blessing Season 2"),
            vec![],
        );

        let final_season = candidate(
            2002,
            "Fruits Basket: The Final",
            Some("Fruits Basket The Final Season"),
            vec![],
        );

        let fan_letter = candidate(
            2003,
            "ONE PIECE FAN LETTER",
            Some("ONE PIECE FAN LETTER"),
            vec![],
        );

        assert!(should_skip_candidate(&season_two, &profile, &seen_ids));
        assert!(should_skip_candidate(&final_season, &profile, &seen_ids));
        assert!(should_skip_candidate(&fan_letter, &profile, &seen_ids));
    }

    #[test]
    fn continuation_lookup_only_uses_active_or_completed_statuses() {
        assert!(should_fetch_continuations_for_status("completed"));
        assert!(should_fetch_continuations_for_status("watching"));
        assert!(!should_fetch_continuations_for_status("planned"));
        assert!(!should_fetch_continuations_for_status("dropped"));
    }

    #[test]
    fn continuation_source_limit_stays_small_to_avoid_anilist_rate_limit() {
        assert_eq!(MAX_CONTINUATION_SOURCE_ANIME, 8);
    }

    #[test]
    fn recommendation_lookup_fetches_extra_pages_when_first_page_is_exhausted() {
        assert_eq!(ANILIST_MAX_PAGES_PER_SIGNAL, 2);
    }

    #[test]
    fn cached_recommendations_become_invalid_when_everything_is_already_in_user_list() {
        let profile = profile(&[1, 2], &[]);

        let cached = serde_json::to_string(&vec![
            RecommendationResponse {
                anilist_id: 1,
                title_romaji: "Anime One".to_string(),
                title_english: None,
                genres: vec!["Action".to_string()],
                tags: Vec::new(),
                cover_image_url: Some("https://example.com/1.jpg".to_string()),
                episodes: Some(12),
                description: None,
            },
            RecommendationResponse {
                anilist_id: 2,
                title_romaji: "Anime Two".to_string(),
                title_english: None,
                genres: vec!["Drama".to_string()],
                tags: Vec::new(),
                cover_image_url: Some("https://example.com/2.jpg".to_string()),
                episodes: Some(12),
                description: None,
            },
        ])
        .expect("cache should serialize");

        assert!(parse_cached_recommendations(&cached, &profile).is_none());
    }

    #[test]
    fn does_not_skip_unrelated_standalone_candidate() {
        let profile = profile(&[16498], &["attack on titan"]);
        let seen_ids = HashSet::new();
        let candidate = candidate(1003, "Dungeon Meshi", Some("Delicious in Dungeon"), vec![]);

        assert!(!should_skip_candidate(&candidate, &profile, &seen_ids));
    }
}
