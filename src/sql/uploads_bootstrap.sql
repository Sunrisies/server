-- Additive prerequisite for the managed upload lifecycle; safe on an existing installation.
CREATE TABLE IF NOT EXISTS uploads (
    id BIGSERIAL PRIMARY KEY,
    uuid VARCHAR NOT NULL UNIQUE,
    url VARCHAR NOT NULL,
    key VARCHAR NOT NULL,
    filename VARCHAR NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
