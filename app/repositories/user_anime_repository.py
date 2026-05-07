from sqlalchemy.orm import Session
from sqlalchemy import select

from app.models.user_anime import UserAnime


class UserAnimeRepository:

    def __init__(self, db: Session):
        self.db = db

    def get_by_id(self, entry_id: int) -> UserAnime | None:
        return self.db.scalar(
            select(UserAnime).where(UserAnime.id == entry_id)
        )

    def get_by_user_and_anime(self, user_id: int, anime_id: int) -> UserAnime | None:
        # Busca o registro específico de um usuário pra um anime
        return self.db.scalar(
            select(UserAnime).where(
                UserAnime.user_id == user_id,
                UserAnime.anime_id == anime_id,
            )
        )

    def get_all_by_user(self, user_id: int) -> list[UserAnime]:
        # Retorna toda a lista de um usuário
        return list(
            self.db.scalars(
                select(UserAnime).where(UserAnime.user_id == user_id)
            )
        )

    def save(self, entry: UserAnime) -> UserAnime:
        self.db.add(entry)
        self.db.commit()
        self.db.refresh(entry)
        return entry

    def delete(self, entry: UserAnime) -> None:
        self.db.delete(entry)
        self.db.commit()
