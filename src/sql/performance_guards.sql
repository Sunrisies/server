-- Run before starting a runtime DB user without CREATE privileges.
-- Existing duplicate view records must be reconciled before creating this index.
CREATE UNIQUE INDEX IF NOT EXISTS post_views_daily_unique ON post_views(post_id, ip, viewed_date);
CREATE TABLE IF NOT EXISTS login_limits (
    key_hash VARCHAR(64) PRIMARY KEY,
    attempts INTEGER NOT NULL,
    expires_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS login_limits_expiry ON login_limits(expires_at);
