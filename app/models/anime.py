from datetime import datetime

from sqlalchemy import String, Text, Integer, DateTime
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base


class Anime(Base):
    __tablename__ = "anime_cache"

    # Internal primary key
    id: Mapped[int] = mapped_column(primary_key=True, index=True)

    # AniList identifier — used to fetch and refresh data from the API
    anilist_id: Mapped[int] = mapped_column(Integer, unique=True, index=True, nullable=False)

    # Titles — AniList provides multiple versions
    title_romaji: Mapped[str] = mapped_column(String(255), nullable=False)
    title_english: Mapped[str | None] = mapped_column(String(255), nullable=True)
    title_native: Mapped[str | None] = mapped_column(String(255), nullable=True)

    # Basic info
    episode_count: Mapped[int | None] = mapped_column(Integer, nullable=True)
    cover_image_url: Mapped[str | None] = mapped_column(String(500), nullable=True)
    description: Mapped[str | None] = mapped_column(Text, nullable=True)

    # Classification — stored as comma-separated strings for simplicity
    # e.g. genres = "Action,Adventure,Fantasy"
    genres: Mapped[str | None] = mapped_column(Text, nullable=True)
    tags: Mapped[str | None] = mapped_column(Text, nullable=True)

    # AniList demographic category (Shounen, Seinen, Shoujo, Josei, Kids)
    demographic: Mapped[str | None] = mapped_column(String(50), nullable=True)

    # Cache control — used to decide when to refresh data from AniList
    cached_at: Mapped[datetime] = mapped_column(DateTime, nullable=False)