from sqlalchemy import ForeignKey, Integer, Text, DateTime
from sqlalchemy.orm import Mapped, mapped_column
from datetime import datetime, timezone

from app.db.base import Base


class RecommendationCache(Base):
    __tablename__ = "recommendation_cache"

    id: Mapped[int] = mapped_column(primary_key=True, index=True)
    user_id: Mapped[int] = mapped_column(Integer, ForeignKey("users.id"), nullable=False, unique=True, index=True)
    data: Mapped[str] = mapped_column(Text, nullable=False)
    cached_at: Mapped[datetime] = mapped_column(DateTime, nullable=False)
