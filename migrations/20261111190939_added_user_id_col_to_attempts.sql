-- Add migration script here
ALTER TABLE google_login_attempts ADD COLUMN user_id UUID DEFAULT NULL REFERENCES users(id) ON DELETE CASCADE;