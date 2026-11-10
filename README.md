# Lorekeeper Backend

A place to save your fictional world so you do not forget.

Writers discuss their worlds through chat. A background researcher saves stated lore in Memgraph.
Later replies search that lore. Writers can inspect sources, browse connections, and resolve supported conflicts through the API.
The product client and SaaS launch workflows remain unfinished.

Updated: 2026-11-03.

## Documentation

- [Product vision](docs/product_soul.md): audience and purpose.
- [User stories](docs/user_stories.md): all 15 required epics.
- [Lifecycle design](docs/design.md): Define, Design, Build, Test, Deploy, and Maintain for each epic.
- [Project plan](docs/project_plan.md): built foundations and remaining scope.
- [Current sprint](docs/current_sprint.md): current evidence and next work.
- [Research retries](docs/research_retry.md), [logging](docs/logging.md), and [tests](tests/README.md).

No epic has recorded acceptance for its full deployed user workflow.

## Data flow

1. Search the world's lore using text search, vector search, and connected entities.
2. Stream a chat reply through SSE. The chat agent can search lore and conflicts but cannot write lore.
3. Save the user message and assistant reply together in Postgres. Each saved message has a UUID.
4. Submit a DBOS research workflow using the user message ID as its original workflow ID.
5. The researcher searches, reads, updates the graph, and reports conflicts through scoped tools.
6. DBOS stores model and tool checkpoints in Postgres. Replayed completed steps reuse their saved results.

A checkpoint is a saved operation result. It does not make a Memgraph write atomic with its Postgres checkpoint.
Chat storage and workflow submission also use separate commits.
Retrying failed research can fork a new workflow. A mapping links the source message to the current attempt.
See [retry behavior and limits](docs/research_retry.md).

The writer controls canon. Supported conflict decisions keep the existing claim or accept the stored replacement.
Accepting a replacement updates an existing entity property and resolves the conflict in one graph transaction.
Core fields and relationship conflict targets still need broader support.

## Stack and layout

| Area | Implementation |
| --- | --- |
| HTTP | Axum 0.8, Tokio, session cookies |
| Relational storage | Postgres 18, SQLx 0.8 |
| Lore graph | Memgraph, neo4rs 0.8 |
| Models | Rig 0.44 through OpenRouter |
| Durable research | DBOS 0.5, using Postgres |
| Cleanup | tokio-cron-scheduler 0.15 and persisted world cleanup jobs |
| Logs | Structured tracing events with private payloads excluded |

```text
src/lib.rs                  shared app modules used by the binary and integration tests
src/app/                    bootstrap, DBOS registration, routes, extractors, middleware
src/service/                auth, worlds, conversations, cleanup
src/service/ai/              chat factory, researcher loop, retrieval, scoped tools
src/data/                   models and Postgres/Memgraph repositories
src/shared/                 validated text types and pagination types
src/infrastructure/         settings, connections, prompt loading, tracing
prompts/agents.md            chat instructions
prompts/researcher.md        lore research instructions
migrations/                 application Postgres schema
.sqlx/                      offline metadata for SQLx macros
tests/                      pure tests, local integration tests, paid smoke evidence
playground/                 chat/stream inspector and API workbench; not the product client
```

GraphRepository lives in `src/data/repositories/graph/mod.rs`.
Its constructor builds private entity, relationship, conflict, graph-conversation, and maintenance repositories.
Children own queries; the parent coordinates graph pages and shared transactions.
Services and tools keep using the existing GraphRepository API.

UserRepository lives in `src/data/repositories/user/mod.rs` and constructs its private auth-token repository.
It owns transactions that consume a token and reset a password or verify an email.
AuthService handles hashing and email delivery without its own database pool.

Model choices live in [app.rs](src/app/app.rs). The old structured extraction service and prompts are removed.

## Local setup

Requires stable Rust, Docker, SQLx CLI, and an OpenRouter key.
Install the CLI with `cargo install sqlx-cli --no-default-features --features postgres`.

```sh
docker compose up -d app-db app-memgraph
```

Postgres uses host port 5433. Memgraph uses Bolt port 7687.
Create a git-ignored `.env`:

```dotenv
DATABASE_URL=postgresql://dev:password@localhost:5433/app_db
OPENROUTER_API_KEY=your-key
MEMGRAPH_URI=bolt://localhost:7687
MEMGRAPH_USER=
MEMGRAPH_PASSWORD=
CORS_ALLOWED_ORIGINS=http://localhost:5173
ENV=dev
API_BASE_URL=http://localhost:8000/api
```

Export settings before starting (the API applies pending migrations during bootstrap):

```sh
set -a
source .env
set +a
SQLX_OFFLINE=true cargo run
```

The app does not load `.env` itself. Bootstrap applies pending application migrations before creating services and workers. Startup stops if migration fails.
DBOS initializes its own system schema during startup.
The app creates graph constraints and indexes during bootstrap.
Non-secret settings live in [config.yaml](src/app/config/config.yaml).
Check `GET http://localhost:8000/health` for `ok`.
With ENV=dev, build the playground using npm install and npm run build inside playground, then open http://localhost:8000/.
Axum serves playground/dist and adds a dev-only /api alias for its client requests.
Staging and production do not expose these static files or that alias.
See [playground instructions](playground/README.md) for static builds and optional Vite live reload.
Ctrl+C drains HTTP requests, then shuts down the cleanup scheduler and DBOS.
SIGTERM behavior still needs implementation or verification before deployment.

## Container build

The Dockerfile has separate playground, Rust build, and runtime stages.
The final image runs as a non-root user and contains no Rust or Node build tools.
The image defaults to ENV=prod. Local Compose explicitly overrides it with ENV=dev.
Built playground files remain available only when ENV=dev.

```sh
docker build -t lorekeeper-backend:local .
```

For the local Compose stack, start the databases first. The API applies pending migrations during bootstrap.
Then run:

```sh
docker compose up -d --build api
```

Compose supplies database service names inside the shared network.
API startup applies embedded application migrations. The SQLx command from local setup is also available for manual migration.
No OpenRouter key or database credentials are needed during image compilation.
The build uses SQLX_OFFLINE and checked-in .sqlx metadata.
.dockerignore excludes local secrets, build output, Git state, and test artifacts from the context.
The container uses SIGINT on stop to match the server's current graceful-shutdown handler.

## API

Protected routes require the session_id cookie. Recovery requests and token confirmations also work without a login.

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/health` | Health response |
| POST | `/auth/signup`, `/auth/login`, `/auth/logout` | Account session actions |
| GET | `/auth/me` | Current user |
| POST | `/auth/password_reset/request` | Request reset by email without a session |
| POST | `/auth/email_verification/request` | Send verification email for the signed-in user |
| POST | `/auth/password_reset` | Confirm reset with query user_id/token and JSON new_password |
| POST | `/auth/email_verification` | Confirm verification with user_id and token |
| GET / POST | `/world` | List or create worlds |
| GET / PATCH / DELETE | `/world/{world_id}` | Read, update, or delete a world |
| GET / POST | `/world/{world_id}/conversations` | List or create conversations |
| GET / DELETE | `/world/{world_id}/conversations/{id}` | Read saved history or delete a conversation |
| POST | `/world/{world_id}/conversations/{id}/messages` | Stream chat; body: `{ "content": "..." }` |
| GET | `/world/{world_id}/graph` | Bounded graph page |
| GET | `/world/{world_id}/entities` | Filtered entity page |
| GET | `/world/{world_id}/entities/search` | Text search for browsing |
| GET | `/world/{world_id}/entities/{entity_id}` | Entity details |
| GET | `/world/{world_id}/entities/{entity_id}/neighbors` | Filtered, paginated connections |
| GET | `/world/{world_id}/relationships/{relationship_id}` | Directed relationship details |
| GET | `/world/{world_id}/conflicts` | Filtered conflict page |
| GET | `/world/{world_id}/conflicts/{conflict_id}` | Conflict claims and sources |
| POST | `/world/{world_id}/conflicts/{conflict_id}/resolve` | Apply the stored conflict decision |
| GET | `/world/{world_id}/messages/{message_id}` | Read a source message with conversation context IDs |
| GET | `/world/{world_id}/messages/{message_id}/research` | Read current research status |
| POST | `/world/{world_id}/messages/{message_id}/research/retry` | Submit missing research or retry eligible failed work |

Conversation read, delete, and chat routes are nested under their world.
The service checks both conversation ownership and the requested world before conversation actions.
The former /conversation/{id} routes are removed.

Query and response types live in [graph.rs](src/data/models/graph.rs) and [conversations.rs](src/data/models/conversations.rs).
Cursors are opaque strings. Graph pages contain edges whose endpoints both belong to that page.
Neighbor pagination counts edges. Agent retrieval uses graph RAG; entity browsing uses text search.

Chat returns Rig events as an SSE stream. Event shapes can change when Rig changes.
Stream errors use `event: error`. An early model failure does not save a partial turn.
A later research submission failure can occur after chat messages save. Check research status before recovery.
Research statuses are `not_submitted`, `queued`, `running`, `completed`, `failed`, and `cancelled`.

## Verification

```sh
SQLX_OFFLINE=true cargo check --offline --all-targets
SQLX_OFFLINE=true cargo test --offline --all-targets
```

Offline metadata validates macro queries against recorded schema information.
Runtime SQLx queries still need real database tests. Refresh metadata when macro queries change.
Use the [playground](playground/README.md) to exercise graph, conflict, source, and recovery APIs manually.
Its API workbench shows status, timing, request paths, and raw JSON.
See [test instructions](tests/README.md) for isolated database runs and paid-test safeguards.

Latest recorded results on 2026-11-03:

- All-target compile check passes with warnings.
- A real Playwright canon regression passes 10 steps, including same-thread and fresh-thread answers after conflict acceptance.
- 44 free native regressions pass, including a local-model request-context check.
- Thirteen selected nested conversation route/access/validation HTTP tests pass.
- Earlier research retry/status verification passes nine selected HTTP tests.
- The broader unit run has 11 passes and one duplicate-row retrieval-ranking failure.

Resolved writer decisions now reach chat through retrieval context, and question-only research preserves accepted lore in that scenario.
Description summaries can still lag behind property changes.
These are scoped results, not a claim that every product workflow passes.
See [test findings](tests/REPORT.md) for evidence and earlier runs.

## Logging and remaining scope

Use `RUST_LOG=info,lorekeeper_backend=debug` for application detail.
See [logging guidance](docs/logging.md) for IDs, privacy, and replay behavior.

Remaining work includes product client flows, long conversation behavior, deletion coordination,
recovery commit gaps, and broader conflict targets. OAuth, account recovery, Stripe billing,
usage controls, service emails, exports, support, and deployment recovery remain required scope.
The [project plan](docs/project_plan.md) tracks these without marking whole epics complete from backend routes alone.

### Auth email confirmation

Password-reset requests accept JSON `{ "email": "user@example.com" }` without a session.
They return the same neutral message for known and unknown accounts.
Verification email requests require a signed-in session and no body.
Verification confirmation accepts JSON `user_id` and `token` without a session.
Password-reset confirmation takes `user_id` and `token` in the query string.
Its JSON body contains `new_password`, using the signup password rules.
Confirmation returns `UserResponse`; sending returns a JSON `message`.
Confirmation does not create a login session. Password reset revokes all sessions for that user.
Sessions live in the application sessions table. Only SHA-256 token hashes are stored.
Login sets an HttpOnly, SameSite=Lax cookie at Path=/, with Secure enabled in production.
Both cookie and database expiry use a fixed 24-hour lifetime. Logout deletes the row and clears the cookie.
SESSION_SECRET_KEY is no longer required. Existing Tower Sessions cookies require a new login.

Email URLs use the required API_BASE_URL setting. Include any path prefix, such as /api for the dev alias.
For production, set the public HTTPS API URL. A trailing slash is optional.
The browser confirmation page remains pending.
Opening an email URL does not consume its token: confirmation requires POST.
The logged-out recovery API workflow passes with local email capture.
See [HTTP verification](tests/artifacts/auth-recovery-http-2026-11-08/summary.md).

### Rust-analyzer performance

Workspace settings use four analysis workers and two Cargo jobs.
The IDE skips expansion of tracing's instrument macro; Cargo still expands and checks it.
SQLx and Askama expansion remain enabled. Unlinked-file diagnostics remain enabled.
Run Developer: Reload Window after changing worker settings to rebuild the analyzer's state.
The syntax cache is 64 trees. This is a tuning choice, not a hard memory limit.

Development builds use incremental compilation and line-table debug information.
This retains source-line backtraces but omits full debugger variable/type details.
Cargo defaults to two jobs to limit memory pressure. Release settings remain unchanged.
Rust-analyzer uses its own build cache, so its first build also needs dependency compilation.
Changing profiles causes one dependency rebuild. Do not clear target directories during normal development.

The binary and integration tests import src/lib.rs instead of compiling separate copies of the app modules.
Startup profiling is temporarily enabled in server.extraEnv. A full window reload loads these environment variables.
Remove RA_PROFILE and RA_LOG after capturing the remaining startup delay.
