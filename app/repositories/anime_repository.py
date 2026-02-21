from sqlalchemy.orm import Session
from sqlalchemy import select

from app.models.anime import Anime


class AnimeRepository:

    def __init__(self, db: Session):
        self.db = db

    def get_by_anilist_id(self, anilist_id: int) -> Anime | None:
        return self.db.scalar(
            select(Anime).where(Anime.anilist_id == anilist_id)
        )

    def save(self, anime: Anime) -> Anime:
        self.db.add(anime)
        self.db.commit()
        self.db.refresh(anime)
        return anime