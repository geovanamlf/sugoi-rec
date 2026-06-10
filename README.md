# Sugoi Rec

Anime list and recommendation app.

The project uses a Rust backend, PostgreSQL, AniList integration, and a React frontend.

## Run

**Backend:**

```bash
cd backend-rust
cp .env.example .env
# edit .env with DATABASE_URL, JWT_SECRET_KEY and FRONTEND_URL
cargo run
```

The backend runs by default at:

```bash
http://127.0.0.1:8080
```

**Frontend:**

```bash
cd frontend
npm install
npm run dev
```

The frontend runs by default at:

```bash
http://localhost:5173
```

## Checks

**Backend:**

```bash
cd backend-rust
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

**Frontend:**

```bash
cd frontend
npm run build
```
