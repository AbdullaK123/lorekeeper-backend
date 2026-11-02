# Research retries

`POST /world/{world_id}/messages/{message_id}/research/retry`

Requires a logged-in owner and a saved user message. No request body is needed.
Returns HTTP 200 with the current research status after accepting the submission.
Poll `GET /world/{world_id}/messages/{message_id}/research` for progress.

- No workflow: submit the saved user text with only earlier conversation messages.
- DBOS ERROR: fork the current attempt from its last failure and save the new workflow ID.
- Recovery attempts exhausted: resume the current workflow with its existing ID.
- Queued, running, completed, or cancelled: return HTTP 409 without starting more work.
- Assistant/tool message: return HTTP 422. Missing or foreign message: HTTP 404.
- Submission or database failure: return HTTP 500.

Postgres advisory locks serialize retries for the same message across app instances.
The mapping table records the current workflow ID; older messages fall back to their message ID.
Apply SQLx migrations before using these endpoints.

DBOS submission and mapping writes remain separate commits. A process crash between them can leave an untracked fork.
This endpoint does not promise exactly-once graph writes or cancel research when a conversation is deleted.
Mapping rows currently have no foreign key because messages live in JSONB; deletion does not remove those rows.
Message access checks still prevent reading research status after its conversation is deleted.
