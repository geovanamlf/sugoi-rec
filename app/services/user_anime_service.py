from sqlalchemy.orm import Session

from app.models.user_anime import UserAnime
from app.repositories.user_anime_repository import UserAnimeRepository
from app.schemas.user_anime import UserAnimeCreate, UserAnimeUpdate


class UserAnimeAlreadyExistsError(Exception):
    pass


class UserAnimeNotFoundError(Exception):
    pass


def add_anime(user_id: int, data: UserAnimeCreate, db: Session) -> UserAnime:
    repo = UserAnimeRepository(db)

    existing = repo.get_by_user_and_anime(user_id, data.anime_id)
    if existing:
        raise UserAnimeAlreadyExistsError("Anime already in list.")

    entry = UserAnime(
        user_id=user_id,
        anime_id=data.anime_id,
        status=data.status,
        rating=data.rating,
        is_favorite=data.is_favorite,
    )

    return repo.save(entry)


def update_anime(user_id: int, anime_id: int, data: UserAnimeUpdate, db: Session) -> UserAnime:
    repo = UserAnimeRepository(db)

    entry = repo.get_by_user_and_anime(user_id, anime_id)
    if not entry:
        raise UserAnimeNotFoundError("Anime not found in list.")

    if data.status is not None:
        entry.status = data.status
    if data.rating is not None:
        entry.rating = data.rating
    if data.is_favorite is not None:
        entry.is_favorite = data.is_favorite

    return repo.save(entry)


def list_anime(user_id: int, db: Session) -> list[dict]:
    repo = UserAnimeRepository(db)
    return repo.get_all_by_user(user_id)


def remove_anime(user_id: int, anime_id: int, db: Session) -> None:
    repo = UserAnimeRepository(db)

    entry = repo.get_by_user_and_anime(user_id, anime_id)
    if not entry:
        raise UserAnimeNotFoundError("Anime not found in list.")

    repo.delete(entry)