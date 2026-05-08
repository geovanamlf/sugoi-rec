from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.api.routers import auth, anime, user_anime, analytics, recommendations

app = FastAPI()

# Permite requisições do frontend React
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:5173", "http://localhost:5174", "http://localhost:5175", "http://localhost:5176"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.include_router(auth.router)
app.include_router(anime.router)
app.include_router(user_anime.router)
app.include_router(analytics.router)
app.include_router(recommendations.router)

@app.get("/health")
def health():
    return {"status": "ok"}
