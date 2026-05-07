from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session

from app.api.deps import get_db
from app.api.deps_auth import get_current_user
from app.models.user import User
from app.schemas.user_anime import UserAnimeCreate, UserAnimeUpdate, UserAnimeResponse
from app.services import user_anime_service
from app.services.user_anime_service import UserAnimeAlreadyExistsError, UserAnimeNotFoundError

router = APIRouter(prefix="/list", tags=["list"])


@router.post("/", response_model=UserAnimeResponse)
def add_anime(
    data: UserAnimeCreate,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    try:
        return user_anime_service.add_anime(current_user.id, data, db)
    except UserAnimeAlreadyExistsError:
        raise HTTPException(status_code=400, detail="Anime already in list.")


@router.get("/", response_model=list[UserAnimeResponse])
def list_anime(
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    return user_anime_service.list_anime(current_user.id, db)


@router.patch("/{anime_id}", response_model=UserAnimeResponse)
def update_anime(
    anime_id: int,
    data: UserAnimeUpdate,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    try:
        return user_anime_service.update_anime(current_user.id, anime_id, data, db)
    except UserAnimeNotFoundError:
        raise HTTPException(status_code=404, detail="Anime not found in list.")


@router.delete("/{anime_id}", status_code=204)
def remove_anime(
    anime_id: int,
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    try:
        user_anime_service.remove_anime(current_user.id, anime_id, db)
    except UserAnimeNotFoundError:
        raise HTTPException(status_code=404, detail="Anime not found in list.")
