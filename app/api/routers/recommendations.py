from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy.orm import Session

from app.api.deps import get_db
from app.api.deps_auth import get_current_user
from app.models.user import User
from app.services import recommendation_service
from app.core.exceptions import AniListUnavailableError, AniListRateLimitError

router = APIRouter(prefix="/recommendations", tags=["recommendations"])


@router.get("/")
def get_recommendations(
    refresh: bool = Query(default=False),
    db: Session = Depends(get_db),
    current_user: User = Depends(get_current_user),
):
    try:
        return recommendation_service.get_recommendations(current_user.id, db, force_refresh=refresh)
    except AniListRateLimitError as e:
        raise HTTPException(status_code=429, detail=str(e))
    except AniListUnavailableError:
        raise HTTPException(status_code=503, detail="AniList is currently unavailable.")