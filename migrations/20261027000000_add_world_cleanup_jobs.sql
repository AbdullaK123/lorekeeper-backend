CREATE TABLE world_cleanup_jobs (
    -- No foreign key: the job must survive deletion of the world.
    world_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
