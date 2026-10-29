-- Assign IDs once in storage. Never generate replacement IDs while reading history.
-- Keep existing IDs, message content, and array order unchanged.
UPDATE conversations
SET messages = (
    SELECT jsonb_agg(
        CASE WHEN message->>'id' IS NULL
            THEN message || jsonb_build_object('id', uuidv7())
            ELSE message
        END ORDER BY position
    )
    FROM jsonb_array_elements(messages) WITH ORDINALITY AS items(message, position)
)
WHERE EXISTS (
    SELECT 1
    FROM jsonb_array_elements(messages) AS message
    WHERE message->>'id' IS NULL
);
