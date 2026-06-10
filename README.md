# Sugoi Rec

Anime list and recommendation app.

The project uses a Rust backend, PostgreSQL, SQLx migrations, AniList integration, and a React frontend.

## Run

**Database:**

Create a PostgreSQL database matching the `DATABASE_URL` configured in `backend-rust/.env`.

The Rust backend applies SQLx migrations automatically on startup.

**Backend:**

```bash
cd backend-rust
cp .env.example .env
# edit .env with DATABASE_URL, JWT_SECRET_KEY and FRONTEND_URL
cargo run
```

The backend runs by default at:

```text
http://127.0.0.1:8080
```

**Frontend:**

```bash
cd frontend
npm install
npm run dev
```

The frontend runs by default at:

```text
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
