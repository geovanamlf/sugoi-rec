from sqlalchemy import ForeignKey, String, Integer, Boolean, DateTime
from sqlalchemy.orm import Mapped, mapped_column
from datetime import datetime, timezone

from app.db.base import Base


class UserAnime(Base):
    __tablename__ = "user_anime"

    id: Mapped[int] = mapped_column(primary_key=True, index=True)

    # Chaves estrangeiras — liga o registro ao usuário e ao anime
    user_id: Mapped[int] = mapped_column(Integer, ForeignKey("users.id"), nullable=False, index=True)
    anime_id: Mapped[int] = mapped_column(Integer, ForeignKey("anime_cache.id"), nullable=False, index=True)

    # Status do anime pra esse usuário
    # Valores possíveis: watching, completed, dropped, planned
    status: Mapped[str] = mapped_column(String(20), nullable=False)

    # Nota de 1 a 10, opcional
    rating: Mapped[int | None] = mapped_column(Integer, nullable=True)

    # Se é favorito ou não
    is_favorite: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)

    # Quando foi adicionado
    added_at: Mapped[datetime] = mapped_column(
        DateTime, default=lambda: datetime.now(timezone.utc), nullable=False
    )
