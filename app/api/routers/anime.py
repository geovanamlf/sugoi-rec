from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session

from app.api.deps import get_db
from app.api.deps_auth import get_current_user
from app.models.user import User
from app.schemas.anime import AnimeResponse
from app.services import anime_service
from app.core.exceptions import AnimeNotFoundError, AniListUnavailableError, AniListRateLimitError

router = APIRouter(prefix="/anime", tags=["anime"])


@router.get("/id/{anilist_id}", response_model=AnimeResponse)
def get_by_id(
    anilist_id: int,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    try:
        return anime_service.get_anime_by_anilist_id(anilist_id, db)
    except AnimeNotFoundError:
        raise HTTPException(status_code=404, detail="Anime not found.")
    except AniListRateLimitError:
        raise HTTPException(status_code=429, detail="AniList rate limit exceeded. Try again later.")
    except AniListUnavailableError:
        raise HTTPException(status_code=503, detail="AniList is currently unavailable.")


@router.get("/search", response_model=AnimeResponse)
def search_by_name(
    q: str,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    try:
        return anime_service.get_anime_by_name(q, db)
    except AnimeNotFoundError:
        raise HTTPException(status_code=404, detail="Anime not found.")
    except AniListRateLimitError:
        raise HTTPException(status_code=429, detail="AniList rate limit exceeded. Try again later.")
    except AniListUnavailableError:
        raise HTTPException(status_code=503, detail="AniList is currently unavailable.")