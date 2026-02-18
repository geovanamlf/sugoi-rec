from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from sqlalchemy import select

from app.api.deps import get_db
from app.models.user import User
from app.schemas.user import UserCreate, UserLogin, UserResponse
from app.core.security import hash_password, verify_password

router = APIRouter(prefix="/auth", tags=["auth"])


@router.post("/register", response_model=UserResponse)
def register(user_data: UserCreate, db: Session = Depends(get_db)):
    existing_user = db.scalar(select(User).where(User.email == user_data.email))
    if existing_user:
        raise HTTPException(status_code=400, detail="Email already registered")

    pwd_bytes = user_data.password.encode("utf-8")
    if len(pwd_bytes) > 72:
        raise HTTPException(
            status_code=400,
            detail=f"Password too long for bcrypt ({len(pwd_bytes)} bytes). Max is 72 bytes."
        )

    try:
        password_hash = hash_password(user_data.password)
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))

    new_user = User(email=user_data.email, password_hash=password_hash)

    db.add(new_user)
    db.commit()
    db.refresh(new_user)

    return new_user


@router.post("/login", response_model=UserResponse)
def login(login_data: UserLogin, db: Session = Depends(get_db)):
    user = db.scalar(select(User).where(User.email == login_data.email))

    if not user:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid credentials",
        )

    pwd_bytes = login_data.password.encode("utf-8")
    if len(pwd_bytes) > 72:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Password too long for bcrypt ({len(pwd_bytes)} bytes). Max is 72 bytes."
        )

    if not verify_password(login_data.password, user.password_hash):
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Invalid credentials",
        )

    return user
