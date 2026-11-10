-- Add migration script here
ALTER TABLE users ADD COLUMN IF NOT EXISTS google_sub TEXT UNIQUE;