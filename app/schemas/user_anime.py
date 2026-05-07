from pydantic import BaseModel, field_validator
from datetime import datetime


class UserAnimeCreate(BaseModel):
    # O que o usuário manda ao adicionar um anime à lista
    anime_id: int
    status: str
    rating: int | None = None
    is_favorite: bool = False

    @field_validator("status")
    @classmethod
    def status_must_be_valid(cls, v: str) -> str:
        allowed = {"watching", "completed", "dropped", "planned"}
        if v not in allowed:
            raise ValueError(f"Status must be one of: {allowed}")
        return v

    @field_validator("rating")
    @classmethod
    def rating_must_be_valid(cls, v: int | None) -> int | None:
        if v is not None and not (1 <= v <= 10):
            raise ValueError("Rating must be between 1 and 10")
        return v


class UserAnimeUpdate(BaseModel):
    # Todos os campos opcionais — usuário pode atualizar só o que quiser
    status: str | None = None
    rating: int | None = None
    is_favorite: bool | None = None

    @field_validator("status")
    @classmethod
    def status_must_be_valid(cls, v: str | None) -> str | None:
        if v is None:
            return v
        allowed = {"watching", "completed", "dropped", "planned"}
        if v not in allowed:
            raise ValueError(f"Status must be one of: {allowed}")
        return v

    @field_validator("rating")
    @classmethod
    def rating_must_be_valid(cls, v: int | None) -> int | None:
        if v is not None and not (1 <= v <= 10):
            raise ValueError("Rating must be between 1 and 10")
        return v


class UserAnimeResponse(BaseModel):
    id: int
    anime_id: int
    user_id: int
    status: str
    rating: int | None
    is_favorite: bool
    added_at: datetime

    class Config:
        from_attributes = True
