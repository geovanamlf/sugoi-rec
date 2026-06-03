
API docs available at `http://localhost:8000/docs`

### Run

**Backend:**
```bash
cd sugoi-rec
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
# crie um arquivo .env com DATABASE_URL e JWT_SECRET_KEY
alembic upgrade head
uvicorn app.main:app --reload
```

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```
