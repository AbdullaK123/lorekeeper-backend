# Adversarial backend tests

These tests expose current bugs. Failures are expected until those bugs are fixed.
The tests do not change production code.

Run the pure tests:

```bash
SQLX_OFFLINE=true cargo test --offline --bin lorekeeper-backend
```

Live tests need the local Compose databases, Python 3, and `requests`.
Create a separate Postgres database once:

```bash
docker exec lorekeeper-backend-app-db-1 createdb -U dev lorekeeper_adversarial
```

Run from the repository root:

```bash
set -a
source .env
set +a
export DATABASE_URL=postgresql://dev:password@localhost:5433/lorekeeper_adversarial
sqlx migrate run
SQLX_OFFLINE=true cargo test --offline --test adversarial_live -- --ignored --nocapture --test-threads=1
```

The test server uses port 18000. The existing server stays running.
The harness refuses another database name. Fault injection uses a trigger scoped to one test conversation.
The trigger is removed in `finally`. Do not interrupt the test during fault injection.
AI tests make paid external requests. Set `QA_AI=1` only when those calls are authorized.
The default command skips them. Run live tests serially because the scheduler processes the shared QA job table.
Set `QA_TESTS` to space-separated Python test names to select specific checks.
For example, use `QA_TESTS='ApiTests.test_foreign_world_conversation_creation_is_denied'`.
Graph tests use the local Memgraph database with fresh world IDs.
Graph fixture world IDs start with `qa-graph-`. API fixture worlds belong to fresh `qa-...@example.com` users.

The Rust live target imports application modules because this crate has no library target.
This also discovers the pure tests again. `--ignored` selects only live tests.

Coverage includes:

- Private routes and cross-user access, including a real private-lore leak attempt.
- Failed assistant saves, normal chat persistence, and background lore extraction.
- Conflict detection and preservation of existing canon.
- World deletion, missing resources, malformed input, and blank names.
- Browser preflight for world updates, account discovery, and logout invalidation.
- Concurrent updates to different world fields.
- Forty concurrent message appends without lost writes.
- Real graph retries, world isolation, injection attempts, deletion, and structured data.
- Scheduled world cleanup, including conflicts, job removal, and preservation of another world.
- Retry after a rejected job removal, without blocking other cleanup jobs.
- Ambiguous aliases, lost search context, and ranking with duplicate rows.

The duplicate-row ranking test checks defensive behavior. Current graph queries may return only unique rows.
Blank-name rejection is a proposed API contract. Confirm that rule before fixing its test.

The suite does not prove restart recovery, client disconnect handling, or concurrent chat ordering.
