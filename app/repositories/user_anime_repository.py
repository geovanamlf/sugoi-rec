from sqlalchemy.orm import Session
from sqlalchemy import select

from app.models.user_anime import UserAnime
from app.models.anime import Anime


class UserAnimeRepository:

    def __init__(self, db: Session):
        self.db = db

    def get_by_id(self, entry_id: int) -> UserAnime | None:
        return self.db.scalar(
            select(UserAnime).where(UserAnime.id == entry_id)
        )

    def get_by_user_and_anime(self, user_id: int, anime_id: int) -> UserAnime | None:
        return self.db.scalar(
            select(UserAnime).where(
                UserAnime.user_id == user_id,
                UserAnime.anime_id == anime_id,
            )
        )

    def get_all_by_user(self, user_id: int) -> list[dict]:
        # Faz join e retorna dicts com dados dos dois modelos
        rows = self.db.execute(
            select(UserAnime, Anime)
            .join(Anime, Anime.id == UserAnime.anime_id)
            .where(UserAnime.user_id == user_id)
        ).all()

        result = []
        for user_anime, anime in rows:
            result.append({
                "id": user_anime.id,
                "anime_id": user_anime.anime_id,
                "user_id": user_anime.user_id,
                "status": user_anime.status,
                "rating": user_anime.rating,
                "is_favorite": user_anime.is_favorite,
                "added_at": user_anime.added_at,
                "anime": {
                    "id": anime.id,
                    "anilist_id": anime.anilist_id,
                    "title_romaji": anime.title_romaji,
                    "title_english": anime.title_english,
                    "cover_image_url": anime.cover_image_url,
                    "episode_count": anime.episode_count,
                    "genres": anime.genres,
                }
            })
        return result

    def save(self, entry: UserAnime) -> UserAnime:
        self.db.add(entry)
        self.db.commit()
        self.db.refresh(entry)
        return entry

    def delete(self, entry: UserAnime) -> None:
        self.db.delete(entry)
        self.db.commit()