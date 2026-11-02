CREATE TABLE message_research_workflows (
    -- Messages live inside conversations.messages JSONB, so no message foreign key exists.
    message_id UUID PRIMARY KEY,
    -- DBOS workflow IDs are text, including IDs generated for retry forks.
    workflow_id TEXT NOT NULL
);
