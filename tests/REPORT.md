# Backend test findings — 2026-10-26

## Latest full run — 2026-10-27

| Suite | Pass | Fail | Skipped |
|---|---:|---:|---:|
| Pure logic | 32 | 5 | 0 |
| Live API | 22 | 2 | 0 |
| Live database and worker checks | 11 | 0 | 0 |
| Total distinct tests | 65 | 7 | 0 |

The scheduled cleanup test passes. It removes entities, conflicts, conversation nodes, attached edges, and the cleanup job.
It preserves another world's graph data.
A second new test rejects one Postgres job removal, verifies other jobs complete, then verifies retry against the empty graph succeeds.
All three deletion transaction tests also pass.

Remaining failing tests:

- Shared aliases route relationships to an arbitrary entity.
- An alias shadows another entity's canonical name.
- Retrieval formatting drops neighbor properties.
- Retrieval formatting drops relationship descriptions.
- Duplicate rows inflate ranking. This remains a defensive test.
- Duplicate signup returns 500 instead of 409.
- Blank world names are accepted. Rejection remains a proposed product rule.

Python reports four API failure entries across two test methods because blank-name checks use three inputs.
Automatic approval review initially blocks the paid AI checks. The user then explicitly approves those calls.
All five AI checks pass in a separate run: save failure, normal chat persistence, private-lore isolation, extraction, and canon preservation.
The Rust API harness counts as one wrapper test; the table counts its Python tests individually.
Test data and the isolated database are removed afterward.
This run does not test extraction racing with world deletion or operating-system shutdown signals.

The sections below preserve earlier results and verification history.

Tests use real Postgres, Memgraph, and OpenRouter calls. Production code stays unchanged.
Postgres fixtures use a separate `lorekeeper_adversarial` database.

| Suite | Tests | Pass | Fail |
|---|---:|---:|---:|
| Pure logic | 37 | 32 | 5 |
| Live database checks | 7 | 7 | 0 |
| Live API checks | 23 | 14 | 9 |
| Total | 67 | 53 | 14 |

The blank-name test contains three failing inputs. Python reports 11 API failures across nine test methods.
All 28 original tests pass. Fresh database migrations pass.
Normal chat, background extraction, conflict preservation, logout invalidation, and 40 concurrent message appends pass.
Test graph fixtures and the isolated Postgres database are removed after the final run.

## Fix first

1. **Private lore leaks across users.** User B creates a conversation inside user A's world.
   B then asks the chat agent for A's saved secret. The response contains the secret.
   `test_foreign_world_conversation_cannot_leak_private_lore` proves the full path.
   Direct reads of A's world and conversations correctly reject B.

2. **A failed save still reports a successful chat.** A scoped Postgres trigger rejects the assistant save.
   The stream sends `finalResponse`. History contains only the user message.
   `test_failed_assistant_save_does_not_report_success` reproduces the partial turn.

3. **World deletion fails once conversations exist.** The endpoint returns 500.
   The foreign key has no deletion policy that supports this action.
   `test_world_with_conversations_can_be_deleted` reproduces it.

4. **Browsers cannot update worlds.** The update route uses PATCH.
   Browser preflight lists GET, POST, PUT, DELETE, and OPTIONS only.
   `test_patch_is_allowed_by_browser_cors` reproduces it.

## Lore quality

- A shared alias routes edges to the first entity indexed.
- An alias can take another entity's canonical name in the relationship index.
- Search formatting drops neighbor properties, such as a city's population.
- Search formatting drops relationship descriptions, such as “only during winter.”

These failures have pure tests in `src/service/ai/extraction/tests.rs` and `src/service/ai/retrieval/tests.rs`.

## Other failures

- Login responses reveal whether an email is registered.
- Duplicate signup returns 500 instead of a conflict response.
- Conversation creation under a missing world returns 500 instead of 404.
- Blank world names pass validation. Rejecting them is a proposed product rule.
- Duplicate rows inflate search ranking. This is a defensive test; current queries may return unique rows.

## Limits

This suite does not prove restart recovery, disconnected-client behavior, or concurrent chat ordering.
Model-backed checks are observations from live runs. They are not deterministic unit tests.
The tests intentionally remain red until the relevant behavior changes.

See `tests/README.md` for repeatable commands.

## Ownership fix verification

The service now checks world ownership before conversation creation.
Compilation passes. Three targeted live regression tests pass:

- Foreign-world conversation creation is denied.
- The private-lore leak through a newly created conversation is blocked.
- Conversation creation under a missing world returns 404.

The setup also confirms that owners can still create conversations.
The remaining tests are not rerun during this targeted check.

## Chat service refactor verification

The chat flow now lives in `ConversationService`. The controller formats stream events.
The service receives retrieval, extraction, and the agent factory through its constructor.
The batch-save method now matches the repository's `append_messages` method.

`cargo check --offline --tests` passes with `SQLX_OFFLINE=true`.
Five of six focused live checks pass: three ownership checks, normal chat saving, and background extraction.
Injected save failure leaves no partial messages, confirming the batch update is atomic.
That test still fails because the stream reports success after the save fails.
Extraction also remains scheduled after a failed save. This behavior needs a separate fix.

The failed-save assertion now rejects any false success, including when zero messages save.
Test fixtures are removed after verification.

## Mechanical fixes — 2026-10-27

- CORS now allows PATCH, matching the existing world update route.
- Missing accounts and accounts without a password hash return the same authentication error as wrong passwords.
- Removed unused imports and unused re-exports of empty tool modules.

Three targeted live checks pass: browser PATCH preflight, matching login error responses, and logout invalidation.
Successful signup and login also pass during test setup.
These checks do not assess login timing differences.
The isolated test database is removed after verification.

## World deletion transaction — 2026-10-27

Three repository tests pass against real Postgres:

- Owned-world deletion commits a cleanup job and removes its conversations.
- A foreign user cannot delete the world or queue its cleanup.
- A rejected job INSERT rolls back the world deletion and restores its conversations.

The rollback test uses a pre-existing job to trigger a real primary-key violation.
These tests cover the Postgres transaction. The Memgraph worker remains pending.

## Chat save failure fix — 2026-10-27

The stream now yields `ServiceError` for both persistence and AI failures.
A failed batch save yields an error and stops before extraction or `finalResponse`.
Two targeted live checks pass: injected save failure and normal chat persistence.
The failure check verifies empty history, no success event, and an explicit stream error event.
The isolated test database is removed after verification.
