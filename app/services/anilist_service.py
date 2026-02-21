from datetime import datetime, timezone

import httpx

from app.core.exceptions import AniListRateLimitError, AniListUnavailableError, AnimeNotFoundError

ANILIST_URL = "https://graphql.anilist.co"

ANIME_QUERY = """
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
"""


def _parse_anime_data(data: dict) -> dict:
    media = data["data"]["Media"]

    all_tags = media.get("tags") or []
    demographic_tags = [t["name"] for t in all_tags if t.get("category") == "Demographic"]
    other_tags = [t["name"] for t in all_tags if t.get("category") != "Demographic"]

    genres = ",".join(media.get("genres") or [])
    tags = ",".join(other_tags)
    demographic = demographic_tags[0] if demographic_tags else None

    return {
        "anilist_id": media["id"],
        "title_romaji": media["title"]["romaji"],
        "title_english": media["title"].get("english"),
        "title_native": media["title"].get("native"),
        "episode_count": media.get("episodes"),
        "cover_image_url": (media.get("coverImage") or {}).get("large"),
        "description": media.get("description"),
        "genres": genres or None,
        "tags": tags or None,
        "demographic": demographic,
        "cached_at": datetime.now(timezone.utc),
    }


def fetch_anime_by_id(anilist_id: int) -> dict:
    return _request({"id": anilist_id})


def fetch_anime_by_name(search: str) -> dict:
    return _request({"search": search})


def _request(variables: dict) -> dict:
    try:
        response = httpx.post(
            ANILIST_URL,
            json={"query": ANIME_QUERY, "variables": variables},
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

    if not data.get("data") or not data["data"].get("Media"):
        raise AnimeNotFoundError("Anime not found on AniList.")

    return _parse_anime_data(data)