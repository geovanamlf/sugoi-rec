from fastapi import FastAPI
from app.api.routers import auth, anime, user_anime

app = FastAPI()

app.include_router(auth.router)
app.include_router(anime.router)
app.include_router(user_anime.router)

@app.get("/health")
def health():
    return {"status": "ok"}
