from collections import Counter
from sqlalchemy.orm import Session
from sqlalchemy import select
import httpx
import time

from app.models.user_anime import UserAnime
from app.models.anime import Anime
from app.core.exceptions import AniListUnavailableError, AniListRateLimitError

ANILIST_URL = "https://graphql.anilist.co"

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


def _get_top_genres(user_id: int, db: Session, top_n: int = 3) -> list[str]:
    rows = db.execute(
        select(UserAnime, Anime)
        .join(Anime, Anime.id == UserAnime.anime_id)
        .where(UserAnime.user_id == user_id)
    ).all()

    counter = Counter()
    for _, anime in rows:
        if anime.genres:
            for genre in anime.genres.split(","):
                counter[genre.strip()] += 1

    return [genre for genre, _ in counter.most_common(top_n)]


def _get_top_tags(user_id: int, db: Session, top_n: int = 3) -> list[str]:
    # Pega as tags mais frequentes na lista do usuário
    rows = db.execute(
        select(UserAnime, Anime)
        .join(Anime, Anime.id == UserAnime.anime_id)
        .where(UserAnime.user_id == user_id)
    ).all()

    counter = Counter()
    for _, anime in rows:
        if anime.tags:
            for tag in anime.tags.split(","):
                counter[tag.strip()] += 1

    return [tag for tag, _ in counter.most_common(top_n)]


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
        raise AniListRateLimitError("AniList rate limit exceeded.")

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


def get_recommendations(user_id: int, db: Session) -> list[dict]:
    top_genres = _get_top_genres(user_id, db)
    top_tags = _get_top_tags(user_id, db)

    if not top_genres and not top_tags:
        return []

    already_in_list = _get_user_anilist_ids(user_id, db)
    seen_ids = set()
    recommendations = []

    # Busca por gênero
    for genre in top_genres:
        results = _fetch(RECOMMENDATION_BY_GENRE_QUERY, {"genre": genre, "page": 1})
        for media in results:
            anilist_id = media["id"]
            if anilist_id in already_in_list or anilist_id in seen_ids:
                continue
            seen_ids.add(anilist_id)
            recommendations.append(_parse_media(media))

        # Delay entre requisições pra respeitar o burst limiter
        time.sleep(0.5)

    # Busca por tag
    for tag in top_tags:
        results = _fetch(RECOMMENDATION_BY_TAG_QUERY, {"tag": tag, "page": 1})
        for media in results:
            anilist_id = media["id"]
            if anilist_id in already_in_list or anilist_id in seen_ids:
                continue
            seen_ids.add(anilist_id)
            recommendations.append(_parse_media(media))

        time.sleep(0.5)

    return recommendations