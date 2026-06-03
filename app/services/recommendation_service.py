from collections import Counter
from sqlalchemy.orm import Session
from sqlalchemy import select
import httpx
import time
import json
from datetime import datetime, timezone, timedelta

from app.models.user_anime import UserAnime
from app.models.anime import Anime
from app.models.recommendation_cache import RecommendationCache
from app.core.exceptions import AniListUnavailableError, AniListRateLimitError

ANILIST_URL = "https://graphql.anilist.co"
CACHE_TTL_HOURS = 6

# Threshold mínimo de relevância pra entrar na busca (30%)
RELEVANCE_THRESHOLD = 0.30

# Máximo de queries pra respeitar o rate limit
MAX_QUERIES = 8

RECOMMENDATION_BY_GENRE_QUERY = """
query ($genre: String, $page: Int) {
  Page(page: $page, perPage: 10) {
    media(genre: $genre, type: ANIME, sort: POPULARITY_DESC) {
      id
      title { romaji english }
      genres
      tags { name }
      coverImage { large }
      episodes
      description(asHtml: false)
    }
  }
}
"""

RECOMMENDATION_BY_TAG_QUERY = """
query ($tag: String, $page: Int) {
  Page(page: $page, perPage: 10) {
    media(tag: $tag, type: ANIME, sort: POPULARITY_DESC) {
      id
      title { romaji english }
      genres
      tags { name }
      coverImage { large }
      episodes
      description(asHtml: false)
    }
  }
}
"""


def _get_relevance_scores(user_id: int, db: Session) -> tuple[list[tuple[str, float]], list[tuple[str, float]]]:
    """
    Retorna listas de (nome, score) ordenadas por relevância
    para gêneros e tags separadamente.
    Score = frequência / total de animes na lista
    """
    rows = db.execute(
        select(UserAnime, Anime)
        .join(Anime, Anime.id == UserAnime.anime_id)
        .where(UserAnime.user_id == user_id)
    ).all()

    total = len(rows)
    if total == 0:
        return [], []

    genre_counter = Counter()
    tag_counter = Counter()

    for _, anime in rows:
        if anime.genres:
            for genre in anime.genres.split(","):
                genre_counter[genre.strip()] += 1
        if anime.tags:
            for tag in anime.tags.split(","):
                tag_counter[tag.strip()] += 1

    # Calcula score e filtra pelo threshold
    genre_scores = [
        (genre, count / total)
        for genre, count in genre_counter.most_common()
        if count / total >= RELEVANCE_THRESHOLD
    ]

    tag_scores = [
        (tag, count / total)
        for tag, count in tag_counter.most_common()
        if count / total >= RELEVANCE_THRESHOLD
    ]

    return genre_scores, tag_scores


def _get_user_anilist_ids(user_id: int, db: Session) -> set[int]:
    rows = db.execute(
        select(UserAnime, Anime)
        .join(Anime, Anime.id == UserAnime.anime_id)
        .where(UserAnime.user_id == user_id)
    ).all()

    return {anime.anilist_id for _, anime in rows}


def _fetch(query: str, variables: dict) -> list[dict]:
    try:
        response = httpx.post(
            ANILIST_URL,
            json={"query": query, "variables": variables},
            headers={"Content-Type": "application/json"},
            timeout=10,
        )
    except httpx.RequestError as e:
        raise AniListUnavailableError(f"Could not reach AniList: {e}")

    if response.status_code == 429:
        retry_after = response.headers.get("Retry-After", "60")
        raise AniListRateLimitError(f"AniList rate limit exceeded. Retry after {retry_after}s.")

    if response.status_code != 200:
        raise AniListUnavailableError(f"AniList returned status {response.status_code}.")

    data = response.json()
    return data.get("data", {}).get("Page", {}).get("media", [])


def _parse_media(media: dict) -> dict:
    return {
        "anilist_id": media["id"],
        "title_romaji": media["title"]["romaji"],
        "title_english": media["title"].get("english"),
        "genres": media.get("genres", []),
        "tags": [t["name"] for t in media.get("tags", [])],
        "cover_image_url": (media.get("coverImage") or {}).get("large"),
        "episodes": media.get("episodes"),
        "description": media.get("description"),
    }


def _get_cache(user_id: int, db: Session) -> list[dict] | None:
    cache = db.scalar(
        select(RecommendationCache).where(RecommendationCache.user_id == user_id)
    )

    if not cache:
        return None

    age = datetime.now(timezone.utc) - cache.cached_at.replace(tzinfo=timezone.utc)
    if age > timedelta(hours=CACHE_TTL_HOURS):
        return None

    return json.loads(cache.data)


def _save_cache(user_id: int, data: list[dict], db: Session) -> None:
    cache = db.scalar(
        select(RecommendationCache).where(RecommendationCache.user_id == user_id)
    )

    if cache:
        cache.data = json.dumps(data)
        cache.cached_at = datetime.now(timezone.utc)
    else:
        cache = RecommendationCache(
            user_id=user_id,
            data=json.dumps(data),
            cached_at=datetime.now(timezone.utc),
        )
        db.add(cache)

    db.commit()


def get_recommendations(user_id: int, db: Session, force_refresh: bool = False) -> list[dict]:
    if not force_refresh:
        cached = _get_cache(user_id, db)
        if cached is not None:
            return cached

    genre_scores, tag_scores = _get_relevance_scores(user_id, db)

    if not genre_scores and not tag_scores:
        return []

    already_in_list = _get_user_anilist_ids(user_id, db)
    seen_ids = set()
    recommendations = []
    query_count = 0

    # Intercala tags e gêneros por relevância
    # Ex: tag Yuri 90%, gênero Slice of Life 40%, tag Romance 35%...
    # Assim tags de alta relevância entram antes de gêneros menos relevantes
    combined = []
    for genre, score in genre_scores:
        combined.append(("genre", genre, score))
    for tag, score in tag_scores:
        combined.append(("tag", tag, score))

    # Ordena tudo por score decrescente
    combined.sort(key=lambda x: x[2], reverse=True)

    for kind, name, score in combined:
        if query_count >= MAX_QUERIES:
            break

        if kind == "genre":
            results = _fetch(RECOMMENDATION_BY_GENRE_QUERY, {"genre": name, "page": 1})
        else:
            results = _fetch(RECOMMENDATION_BY_TAG_QUERY, {"tag": name, "page": 1})

        query_count += 1

        for media in results:
            anilist_id = media["id"]
            if anilist_id in already_in_list or anilist_id in seen_ids:
                continue
            seen_ids.add(anilist_id)
            recommendations.append(_parse_media(media))

        time.sleep(2)

    _save_cache(user_id, recommendations, db)

    return recommendations