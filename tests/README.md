# Adversarial backend tests

These tests check current contracts and expose remaining defects.
Updated: 2026-11-02. See [findings](REPORT.md) for the latest scoped results.
The tests do not change production code.

Run the pure tests:

```bash
SQLX_OFFLINE=true cargo test --offline --lib
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
SQLX_OFFLINE=true cargo test --offline --jobs 2 --test adversarial_live -- --ignored --nocapture --test-threads=1 --skip adversarial_live_api --skip researcher_smoke_conversation
```

The command above selects free native database tests. Run the HTTP wrapper separately below.
The HTTP test server uses port 18000. The existing server can stay running on port 8000.
The harness refuses another database name. Fault injection uses a trigger scoped to one test conversation.
The trigger is removed in `finally`. Do not interrupt the test during fault injection.
AI tests make paid external requests. Set `QA_AI=1` only when those calls are authorized.
The command above excludes the paid smoke harness and the HTTP wrapper.
The HTTP wrapper skips its paid Python cases unless QA_AI=1. Run live tests serially because the scheduler processes the shared QA job table.
Set `QA_TESTS` to space-separated Python test names to select specific checks.
For example, use `QA_TESTS='ApiTests.test_foreign_world_conversation_creation_is_denied'`.
Graph tests use the local Memgraph database with fresh world IDs.
Graph fixture world IDs start with `qa-graph-`. API fixture worlds belong to fresh `qa-...@example.com` users.

The Rust live target imports the shared lorekeeper_backend library.
Application unit tests live in the library target and are not copied into the integration target.
`--ignored` selects only live tests.

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
- Contextual identity scenarios, preserved search context, and the known duplicate-row ranking assertion.
- Populated graph/entity/neighbor pagination, filters, direction, and source-message access.
- Atomic supported conflict decisions and exact existing-property targets.
- Research status mappings, current-attempt tracking, retry access, concurrent requests, and saved-history reconstruction.
- Log capture for private request data and internal error payloads.

The duplicate-row ranking test checks defensive behavior. Current graph queries may return only unique rows.
Typed request validation now rejects blank world names with HTTP 422.

DBOS engine restart coverage is described below. OS-level crash recovery, client disconnect handling, and concurrent chat ordering remain unverified.


## Paid conversation smoke tests

The smoke runner accepts a scenario file and retains its synthetic world and evidence.
It waits for each background research run before sending the next turn.
This makes paid chat, research, and embedding calls. Run only with explicit authorization.

After the isolated database setup above, choose a new output directory and log file:

```bash
export QA_SMOKE_PAID=1
export QA_SMOKE_SCENARIO="$PWD/tests/scenarios/ardent_reach_10.json"
export QA_SMOKE_OUTPUT="$PWD/tests/artifacts/my-smoke-run"
export QA_SMOKE_LOG=/tmp/lorekeeper-my-smoke-run.log
export RUST_LOG=info
cargo test --offline --test adversarial_live researcher_smoke_conversation -- --ignored --nocapture --test-threads=1 > "$QA_SMOKE_LOG" 2>&1
```

Without a scenario file, the runner uses the original 20-turn Brinewake scenario.
Use a fresh output directory to preserve earlier evidence.
System-created conflict links have no separate source list; their conflict nodes carry the reporting-message sources.


## DBOS researcher replay tests (no model charges)

With the isolated QA database configured, run:

```bash
cargo test --offline --test adversarial_live durable_research -- --ignored --test-threads=1
```

These tests use a local scripted HTTP provider and real Postgres.
They exercise the production research driver with two tool calls per model turn.
One test shuts down DBOS during a model request, recreates it, and verifies recorded operations do not execute again.
This simulates engine restart in one process; it does not prove graph-write/checkpoint atomicity.
The broader non-AI run must use `--skip researcher_smoke_conversation` to exclude the explicitly paid smoke harness.


## Research status and retry HTTP tests

After exporting the isolated QA environment and applying migrations:

```sh
unset QA_AI QA_SMOKE_PAID
export QA_TESTS='ApiTests.test_research_retry_rejects_ineligible_states ApiTests.test_research_retry_enforces_access_and_user_role ApiTests.test_research_retry_resumes_exhausted_workflow ApiTests.test_research_retry_tracks_fork_and_blocks_duplicate ApiTests.test_research_status_maps_all_stored_states ApiTests.test_research_status_missing_workflow_is_not_submitted ApiTests.test_research_status_enforces_message_access ApiTests.test_research_status_lookup_error_is_not_not_submitted ApiTests.test_research_status_deleted_message_is_not_exposed'
SQLX_OFFLINE=true cargo test --offline --jobs 2 --test adversarial_live adversarial_live_api -- --ignored --test-threads=1
```

These cases use synthetic DBOS records with isolated application versions. They make no model calls.
The lookup-error case uses an unknown stored DBOS status, not a database outage.
The scripted submission case in cases/research_retry.rs runs a real local DBOS workflow without a model.
Fork tests verify current-attempt polling and two concurrent retries producing one accepted fork.

## Latest recorded results — 2026-11-02

- All-target offline compilation passes with warnings.
- 43 free native integration tests pass in 9.17 seconds.
- Nine selected retry/status HTTP tests pass in 9.475 seconds; wrapper takes 10.93 seconds.
- The all-target unit run stops with 11 passes and one duplicate-row retrieval-ranking failure.

These results come from separate commands. They are not a full-suite pass total.
The paid smoke artifacts are historical evidence; this verification makes no paid provider calls.


## Nested conversation routes

Conversation read/delete and chat use /world/{world_id}/conversations/{conversation_id} and its /messages child.
The former /conversation/{id} routes are removed. Test clients and smoke runners pass the world ID explicitly.
The nested-conversation test checks a valid read/delete and rejects wrong or missing parent worlds.
Wrong-world chat checks stop before model retrieval, so these checks incur no model charges.


## Development static serving

Run SQLX_OFFLINE=true cargo test --offline --lib playground::tests:: for the isolated router test.
It creates temporary HTML/JS files and checks dev-only serving, staging/production exclusion,
the dev /api alias, original API routes, missing files, and path traversal.
No database or model calls are needed.


For ApiTests.test_dev_static_playground_preserves_api_sessions, first run npm run build inside playground and set ENV=dev.
The live check loads the real HTML/JS assets and verifies the /api alias preserves sessions and rejects anonymous users.
The full Python suite includes this check, so build static assets before running that suite.


## Container packaging

Build the image with docker build -t lorekeeper-backend:local . from the repository root.
The latest temporary-container check uses the isolated QA database and a dummy provider key.
It verifies prod/dev route gates, health, built assets, non-root execution, absent build tools/secrets, and graceful stop.
This is image validation, not deployed acceptance or a full-suite result. Apply application migrations separately.


## Real Playwright writer workflow (paid)

The opt-in script tests the built playground through Chromium. It submits real chat/research requests.
Run only after authorizing provider charges. Use the isolated QA database and a fresh output directory.
Install Playwright outside the repository if it is not already available:

```sh
npm install --prefix /tmp/lorekeeper-browser --no-save playwright
/tmp/lorekeeper-browser/node_modules/.bin/playwright install chromium
QA_PLAYWRIGHT_PAID=1 \
PLAYWRIGHT_MODULE=/tmp/lorekeeper-browser/node_modules/playwright/index.mjs \
QA_BASE_URL=http://127.0.0.1:18000 \
QA_PLAYWRIGHT_OUTPUT=tests/artifacts/my-browser-run \
node tests/playwright_workflow.mjs
```

Start the dev API against QA data before this command. An existing compatible Chromium can be selected with PLAYWRIGHT_CHROMIUM_EXECUTABLE.
The script retains screenshots, saved messages, and response outcomes. It exits nonzero on failed assertions.
QA_PLAYWRIGHT_RESUME=1 can reuse saved account/world data after harness interruptions.
After the accepted-canon assertion fails, QA_PLAYWRIGHT_FINISH_ONLY=1 with resume checks errors/mobile controls and a fresh conversation.
That continuation still performs a paid chat and retains the original failure result.
The [2026-11-03 evidence](artifacts/playwright-2026-11-03/summary.md) records an actual canon reversion, not a passing acceptance run.


### Canon authority regression

The paid browser script checks the accepted property and exact entity/source snapshot after research finishes.
It repeats the age question in a fresh conversation. The [passing rerun](artifacts/playwright-canon-decisions-2026-11-03/summary.md) preserves age 41.
Free coverage includes resolved-decision formatting and chat_canon_context::, which inspects a local model request for current lore in system context.
The free native suite now contains 44 passing checks. The wider unit duplicate-ranking failure remains a separate historical finding.

### Extensive Playwright mode

Set `QA_PLAYWRIGHT_EXTENSIVE=1` alongside the existing paid-browser flags.
This runs 20 paid chat turns, background research, and two conflict-description rewrite calls.
The harness checks corrected descriptions, repeated acceptance, longer history, reload, mobile reads, and errors.
The normal run also fails if the description omits age 41, retains age 34, or loses Bellhaven/Glasswatch.
Latest evidence: [28 passing checks](artifacts/playwright-extensive-description-2026-11-03/summary.md).

### Auth-token repository checks

Use the migrated, isolated `lorekeeper_adversarial` Postgres database. No email or model key is needed.

```bash
DATABASE_URL=postgresql://dev:password@localhost:5433/lorekeeper_adversarial \
SQLX_OFFLINE=true \
cargo test --offline --jobs 1 --lib data::repositories::user::tests:: -- --ignored --test-threads=1
```

The 17 tests live in `src/data/repositories/user/tests.rs` beside the private token child.
They cover token lifecycles, service password resets, verification, rollback, replay, concurrent password updates, and session-deletion rollback.
The Resend client uses a dummy key in these tests and sends no emails.
[Recorded results](artifacts/auth-tokens-2026-11-07/summary.md).

Latest auth transaction evidence: [17 passing tests and HTTP session checks](artifacts/custom-sessions-2026-11-08/summary.md).

### Auth confirmation request validation

Run `SQLX_OFFLINE=true cargo test --offline --jobs 1 --test adversarial_live auth_requests::`.
These four tests check required user identity, token length, and password policy.
They use no database, model, or email provider.

### Recovery HTTP workflow

`auth_recovery_http::recovery_http_requires_valid_input_and_consumes_links_once` is opt-in.
Use isolated QA database and Memgraph settings from the native integration setup.
Set `RESEND_BASE_URL=http://127.0.0.1:18082` and `RESEND_API_KEY=re_test_not_sent`.
The test binds that capture port itself. Use a synthetic OpenRouter key; no model calls occur.
Set `API_BASE_URL=http://localhost:18000` and the normal session/CORS/dev settings.
Run `SQLX_OFFLINE=true cargo test --offline --jobs 2 --test adversarial_live auth_recovery_http:: -- --ignored --test-threads=1`.
The test runs HTTP requests through the full router in process and captures SDK email requests locally.
Evidence: [passing workflow](artifacts/auth-recovery-http-2026-11-08/summary.md).

The recovery HTTP workflow also checks custom session cookies, profile reads, expired/forged cookie rejection, logout, and reset invalidation across two sessions.
Run with `ENV=dev` and `ENV=prod` to check the Secure flag. No signing-key setting is required.
