from collections import Counter
from sqlalchemy.orm import Session
from sqlalchemy import select

import httpx

from app.models.user_anime import UserAnime
from app.models.anime import Anime
from app.core.exceptions import AniListUnavailableError, AniListRateLimitError

ANILIST_URL = "https://graphql.anilist.co"

# Busca animes por gênero no AniList
RECOMMENDATION_QUERY = """
query ($genre: String, $page: Int) {
  Page(page: $page, perPage: 10) {
    media(genre: $genre, type: ANIME, sort: POPULARITY_DESC) {
      id
      title {
        romaji
        english
      }
      genres
      coverImage {
        large
      }
      episodes
      description(asHtml: false)
    }
  }
}
"""


def _get_top_genres(user_id: int, db: Session, top_n: int = 3) -> list[str]:
    # Pega os gêneros mais frequentes na lista do usuário
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


def _get_user_anilist_ids(user_id: int, db: Session) -> set[int]:
    # IDs dos animes que o usuário já tem na lista — pra não recomendar de novo
    rows = db.execute(
        select(UserAnime, Anime)
        .join(Anime, Anime.id == UserAnime.anime_id)
        .where(UserAnime.user_id == user_id)
    ).all()

    return {anime.anilist_id for _, anime in rows}


def _fetch_by_genre(genre: str) -> list[dict]:
    try:
        response = httpx.post(
            ANILIST_URL,
            json={"query": RECOMMENDATION_QUERY, "variables": {"genre": genre, "page": 1}},
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


def get_recommendations(user_id: int, db: Session) -> list[dict]:
    top_genres = _get_top_genres(user_id, db)

    if not top_genres:
        return []

    already_in_list = _get_user_anilist_ids(user_id, db)

    seen_ids = set()
    recommendations = []

    for genre in top_genres:
        results = _fetch_by_genre(genre)

        for media in results:
            anilist_id = media["id"]

            # Pula se já tá na lista do usuário ou já apareceu nessa busca
            if anilist_id in already_in_list or anilist_id in seen_ids:
                continue

            seen_ids.add(anilist_id)
            recommendations.append({
                "anilist_id": anilist_id,
                "title_romaji": media["title"]["romaji"],
                "title_english": media["title"].get("english"),
                "genres": media.get("genres", []),
                "cover_image_url": (media.get("coverImage") or {}).get("large"),
                "episodes": media.get("episodes"),
                "description": media.get("description"),
            })

    return recommendations
