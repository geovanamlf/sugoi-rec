from datetime import datetime

from pydantic import BaseModel


class AnimeBase(BaseModel):
    anilist_id: int
    title_romaji: str
    title_english: str | None = None
    title_native: str | None = None
    episode_count: int | None = None
    cover_image_url: str | None = None
    description: str | None = None
    genres: str | None = None
    tags: str | None = None
    demographic: str | None = None


class AnimeCreate(AnimeBase):
    # Used internally when saving AniList data to the cache table.
    # cached_at is set by the service, not by the user.
    cached_at: datetime


class AnimeResponse(AnimeBase):
    id: int
    cached_at: datetime

    class Config:
        from_attributes = True