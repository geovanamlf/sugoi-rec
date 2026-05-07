from sqlalchemy.orm import Session
from sqlalchemy import select
from collections import Counter

from app.models.user_anime import UserAnime
from app.models.anime import Anime


def get_genre_stats(user_id: int, db: Session) -> list[dict]:
    # Busca todos os animes da lista do usuário junto com os dados do anime
    rows = db.execute(
        select(UserAnime, Anime)
        .join(Anime, Anime.id == UserAnime.anime_id)
        .where(UserAnime.user_id == user_id)
    ).all()

    # Junta todos os gêneros de todos os animes da lista
    genre_counter = Counter()
    for user_anime, anime in rows:
        if anime.genres:
            for genre in anime.genres.split(","):
                genre_counter[genre.strip()] += 1

    # Retorna ordenado do mais frequente pro menos
    return [{"genre": g, "count": c} for g, c in genre_counter.most_common()]


def get_rating_stats(user_id: int, db: Session) -> dict:
    # Busca só os registros que têm rating
    rows = db.scalars(
        select(UserAnime).where(
            UserAnime.user_id == user_id,
            UserAnime.rating.isnot(None),
        )
    ).all()

    if not rows:
        return {"average_rating": None, "rated_count": 0}

    ratings = [r.rating for r in rows]
    return {
        "average_rating": round(sum(ratings) / len(ratings), 2),
        "rated_count": len(ratings),
    }


def get_status_stats(user_id: int, db: Session) -> list[dict]:
    rows = db.scalars(
        select(UserAnime).where(UserAnime.user_id == user_id)
    ).all()

    counter = Counter(r.status for r in rows)

    # Garante que todos os status aparecem, mesmo com contagem 0
    all_statuses = ["watching", "completed", "dropped", "planned"]
    return [{"status": s, "count": counter.get(s, 0)} for s in all_statuses]
