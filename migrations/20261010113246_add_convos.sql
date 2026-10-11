-- Add migration script here
CREATE TABLE IF NOT EXISTS conversations (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    world_id UUID NOT NULL REFERENCES worlds(id),
    user_id UUID NOT NULL REFERENCES users(id),
    title TEXT NOT NULL DEFAULT '',
    messages JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_conversations_world_id ON conversations(world_id);
CREATE INDEX idx_conversations_user_id ON conversations(user_id);