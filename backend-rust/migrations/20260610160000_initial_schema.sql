CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS ix_users_email
ON users (email);

CREATE INDEX IF NOT EXISTS ix_users_id
ON users (id);

CREATE TABLE IF NOT EXISTS anime_cache (
    id SERIAL PRIMARY KEY,
    anilist_id INTEGER NOT NULL,
    title_romaji VARCHAR(255) NOT NULL,
    title_english VARCHAR(255),
    title_native VARCHAR(255),
    episode_count INTEGER,
    cover_image_url VARCHAR(500),
    description TEXT,
    genres TEXT,
    tags TEXT,
    demographic VARCHAR(50),
    cached_at TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS ix_anime_cache_anilist_id
ON anime_cache (anilist_id);

CREATE INDEX IF NOT EXISTS ix_anime_cache_id
ON anime_cache (id);

CREATE TABLE IF NOT EXISTS user_anime (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    anime_id INTEGER NOT NULL REFERENCES anime_cache(id),
    status VARCHAR(20) NOT NULL,
    rating INTEGER,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    added_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_user_anime_id
ON user_anime (id);

CREATE INDEX IF NOT EXISTS ix_user_anime_user_id
ON user_anime (user_id);

CREATE INDEX IF NOT EXISTS ix_user_anime_anime_id
ON user_anime (anime_id);

CREATE TABLE IF NOT EXISTS recommendation_cache (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    data TEXT NOT NULL,
    cached_at TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_recommendation_cache_id
ON recommendation_cache (id);

CREATE UNIQUE INDEX IF NOT EXISTS ix_recommendation_cache_user_id
ON recommendation_cache (user_id);
