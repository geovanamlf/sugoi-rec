from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session

from app.api.deps import get_db
from app.api.deps_auth import get_current_user
from app.models.user import User
from app.services import analytics_service

router = APIRouter(prefix="/analytics", tags=["analytics"])


@router.get("/genres")
def genre_stats(
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    return analytics_service.get_genre_stats(current_user.id, db)


@router.get("/ratings")
def rating_stats(
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    return analytics_service.get_rating_stats(current_user.id, db)


@router.get("/status")
def status_stats(
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    return analytics_service.get_status_stats(current_user.id, db)
