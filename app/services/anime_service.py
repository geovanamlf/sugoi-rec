from datetime import datetime, timezone, timedelta

from sqlalchemy.orm import Session

from app.models.anime import Anime
from app.repositories.anime_repository import AnimeRepository
from app.services import anilist_service

CACHE_TTL_DAYS = 7


def _is_cache_valid(anime: Anime) -> bool:
    cached_at = anime.cached_at
    if cached_at.tzinfo is None:
        cached_at = cached_at.replace(tzinfo=timezone.utc)
    return datetime.now(timezone.utc) - cached_at < timedelta(days=CACHE_TTL_DAYS)


def get_anime_by_anilist_id(anilist_id: int, db: Session) -> Anime:
    repo = AnimeRepository(db)

    cached = repo.get_by_anilist_id(anilist_id)
    if cached and _is_cache_valid(cached):
        return cached

    data = anilist_service.fetch_anime_by_id(anilist_id)
    return _save_or_update(data, cached, repo)


def get_anime_by_name(search: str, db: Session) -> Anime:
    data = anilist_service.fetch_anime_by_name(search)

    repo = AnimeRepository(db)
    cached = repo.get_by_anilist_id(data["anilist_id"])
    return _save_or_update(data, cached, repo)


def _save_or_update(data: dict, existing: Anime | None, repo: AnimeRepository) -> Anime:
    if existing:
        for key, value in data.items():
            setattr(existing, key, value)
        return repo.save(existing)

    return repo.save(Anime(**data))