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


## Background researcher wiring — 2026-10-29

Chat exposes only `search_world`. Background research starts after successful turn storage.
The researcher loads `researcher.md` and exposes search, full entity reads, conflict reporting, and exact-ID batch writes.
The old extractor, its types, prompts, merge/alias implementation tests, and legacy write methods are removed.
Storage retry tests now exercise exact saved IDs through the active batch writer.
Removing old implementation tests does not establish researcher identity accuracy.

Verification:

- 13 local graph and agent-tool tests pass without model calls.
- These cover tool permissions, full-read details, world isolation, rollback, retries, and relationship identity.
- Pure Rust tests: nine pass, three existing retrieval failures remain.
- Remaining failures: omitted neighbor properties, omitted relationship qualifiers, and duplicate-row ranking inflation.
- Paid model behavior, conversation save-failure integration, durable retries, and deletion races are not re-evaluated here.

The researcher still uses an in-memory spawned task. Restart recovery remains unfinished.


## Realistic 20-turn researcher smoke — 2026-10-29

Completed 20 chat turns and 20 background research runs against real OpenRouter models and local databases.
The retained synthetic world contains 21 entities, two unresolved conflicts, and 36 relationships including conflict links.
No stream errors or reported research failures occurred.
Route retractions, contextual identity, minor facts, unresolved conflicts, and hypothetical isolation behaved correctly in this run.
Conversation-level sources remain inconsistent; message-level citations are not implemented.
This sequential smoke does not test concurrent writes, restart recovery, or recall from a fresh conversation.

See [the detailed result and transcript](artifacts/smoke-2026-10-29/summary.md).


## Stable message IDs — 2026-10-29

User, assistant, and tool messages now store UUIDs. New chat messages receive IDs before storage.
The additive migration assigns missing IDs to existing messages without changing order, content, or existing IDs.
The migration is applied to both local development and QA databases.

Three real Postgres tests pass: migration preservation and repeatability, stable IDs across repeated reads, and concurrent message appends.
No model calls run for this change.


## Second 20-turn smoke with message sources — 2026-10-29

All 20 chat turns and 20 researcher runs complete in a fresh world.
The graph has 23 entities, two unresolved conflicts, and 38 relationships including conflict links.
Every entity and lore relationship has valid user-message sources without duplicates.
Retained records preserve earlier sources across updates. Hypothetical and read-only turns change neither facts nor sources.
Conflict message sources remain outside this change.

See [the result, transcript, and source audit](artifacts/smoke-2026-10-29-run2/summary.md).


## Conflict sources and new 10-turn smoke — 2026-10-29

Conflicts now store trusted reporting-message IDs, append them without duplicates, and preserve legacy reads.
The focused Memgraph test covers source accumulation, retry deduplication, and missing historical source fields.

A fresh Ardent Reach world completes ten chat turns and ten researcher runs without reported errors.
It contains 13 entities, two unresolved conflicts, and 16 lore relationships plus two conflict links.
The conflicts reference the exact user messages from turns 5 and 10.
All entity, conflict, and lore relationship source audits pass.
Hypothetical and recall turns leave the graph unchanged.

See [the result, transcript, and graph](artifacts/smoke-2026-10-29-10turns/summary.md).
