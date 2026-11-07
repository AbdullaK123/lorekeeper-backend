-- Add migration script here
CREATE TABLE IF NOT EXISTS auth_tokens (
    token_hash TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    purpose TEXT NOT NULL CHECK (purpose IN ('password_reset', 'email_verification')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expired_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '5 minutes'),
    UNIQUE (user_id, purpose),
    CHECK (expired_at > created_at)
);

CREATE INDEX IF NOT EXISTS idx_user_id_auth_tokens ON auth_tokens (user_id);

ALTER TABLE users ADD COLUMN verified BOOLEAN NOT NULL DEFAULT FALSE;

