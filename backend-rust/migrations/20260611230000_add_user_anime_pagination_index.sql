CREATE INDEX IF NOT EXISTS idx_user_anime_user_added_at_id
ON user_anime (user_id, added_at DESC, id DESC);
