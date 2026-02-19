from fastapi import FastAPI
from app.api.routers import db_test, auth

app = FastAPI()

app.include_router(db_test.router)
app.include_router(auth.router)

@app.get("/health")
def health():
    return {"status": "ok"}
