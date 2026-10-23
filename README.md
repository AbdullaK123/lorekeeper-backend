# Lorekeeper Backend

**A place to save your fictional world so you don't forget.**

Writers chat about their world. An AI replies like a collaborator, and in the background it saves the lore (characters, places, factions, events) into a knowledge graph. Later answers come from that graph, so they trace back to what the writer actually said.

See [docs/product_soul.md](docs/product_soul.md) for the product vision and [docs/project_plan.md](docs/project_plan.md) for the build plan.

## How it works

One chat turn, `POST /conversation/{id}/messages`:

```
writer's message
   │
   ├─ 1. search the world's graph ───────────┐   (meaning + exact words, merged with RRF,
   │                                         │    then pull in connected entities)
   ├─ 2. chat agent replies ◄── lore ────────┘
   │      • streams back as SSE
   │      • may call the search_world tool for more
   │      • never writes to the graph
   │
   ├─ 3. save the message and the reply (Postgres)
   │
   └─ 4. background job: extract lore ──► Memgraph
          • find entities in the message
          • new → create · existing → check for contradictions, merge the rest
          • every write is safe to run twice
```

Two rules shape the design:

- **Only extraction writes lore.** The chat agent can read the graph but never change it. Extraction runs on **every** message, so lore never depends on the AI deciding to save it.
- **Contradictions don't overwrite.** If a message contradicts saved lore, extraction records a **conflict** and keeps the old value until the writer decides.

## Stack

| Part | Choice |
|---|---|
| HTTP server | [Axum](https://docs.rs/axum) 0.8 |
| Users, worlds, conversations | Postgres 18 via [sqlx](https://docs.rs/sqlx) |
| Knowledge graph + search indexes | [Memgraph](https://memgraph.com/docs) via [neo4rs](https://docs.rs/neo4rs) |
| AI (chat, extraction, embeddings) | [Rig](https://docs.rs/rig-core) 0.44, through [OpenRouter](https://openrouter.ai) |
| Sessions | [tower-sessions](https://docs.rs/tower-sessions), stored in Postgres |

Models are set in [src/app/app.rs](src/app/app.rs): the chat and extraction model, and `openai/text-embedding-3-small` for embeddings (1536 dimensions).

## Project layout

```
src/
├── main.rs                  starts the server
├── app/                     HTTP layer
│   ├── app.rs               builds every service once, wires routes
│   ├── controllers/         route handlers (auth, world, conversation)
│   ├── middleware/          sessions, auth extractor
│   └── config/              env settings + config.yaml (pool sizes)
├── service/                 business logic
│   ├── ai/
│   │   ├── agent.rs         ChatAgentFactory: one agent per request
│   │   ├── extraction.rs    ExtractionService: message → graph
│   │   ├── retrieval.rs     RetrievalService: graph RAG search
│   │   └── tools/           search_world tool for the agent
│   └── auth.rs, world.rs, conversation.rs
├── data/
│   ├── models/              structs: DB rows, graph nodes, extraction types
│   └── repositories/        SQL (Postgres) and Cypher (Memgraph)
└── infrastructure/          DB connections, prompt loading, tracing

prompts/                     system prompts (loaded at startup)
├── agents.md                the chat agent
├── extraction.md            finds entities and relationships in a message
└── conflict.md              compares new info with saved lore

migrations/                  Postgres schema (sqlx)
playground/                  dev UI for testing by hand (see its README)
docs/                        product vision and build plan
```

## Setup

### 1. Prerequisites

- Rust (stable) and Docker
- `sqlx-cli` for migrations: `cargo install sqlx-cli --no-default-features --features postgres`
- An [OpenRouter](https://openrouter.ai) API key

### 2. Start the databases

```bash
docker compose up -d app-db app-memgraph
```

| Service | Host port | Notes |
|---|---|---|
| Postgres (`app-db`) | **5433** | Not 5432, so it can run next to other projects' Postgres. |
| Memgraph (`app-memgraph`) | 7687 | Bolt protocol, used by the app and `mgconsole`. |

### 3. Create `.env`

`.env` is git-ignored. Every value is required.

```bash
DATABASE_URL=postgresql://dev:password@localhost:5433/app_db
OPENROUTER_API_KEY=sk-or-...
MEMGRAPH_URI=bolt://localhost:7687
MEMGRAPH_USER=
MEMGRAPH_PASSWORD=
SESSION_SECRET_KEY=<at least 64 random characters>
CORS_ALLOWED_ORIGINS=http://localhost:3000
ENV=dev
```

| Variable | What it does |
|---|---|
| `DATABASE_URL` | Postgres connection. sqlx also uses it **at compile time** to check every SQL query. |
| `OPENROUTER_API_KEY` | Used for chat, extraction and embeddings. |
| `MEMGRAPH_URI`, `_USER`, `_PASSWORD` | Memgraph connection. The local container has no auth, so user and password stay empty. |
| `SESSION_SECRET_KEY` | Signs the session cookie. Must be 64+ bytes, or the server refuses to start. Generate one with `head -c 96 /dev/urandom \| base64 \| tr -d '\n=+/'`. |
| `CORS_ALLOWED_ORIGINS` | Comma-separated origins allowed to call the API with cookies. |
| `ENV` | `dev`, `staging` or `prod`. `prod` marks the session cookie as HTTPS-only. |

Pool sizes and other non-secret settings live in [src/app/config/config.yaml](src/app/config/config.yaml).

### 4. Run migrations

```bash
sqlx migrate run
```

The Memgraph schema doesn't need a migration. The app creates its constraint and search indexes at every startup, and that's safe to repeat.

### 5. Run

The app reads settings from environment variables only. It doesn't load `.env` itself, so export it first:

```bash
set -a; . ./.env; set +a
cargo run
```

You should see `Started server on port 8000...`. Check it with `curl localhost:8000/health`, which should return `ok`.

## API

All routes except signup, login and health need a session cookie from `POST /auth/login`.

| Method | Path | Body | Returns |
|---|---|---|---|
| GET | `/health` | | `ok` |
| POST | `/auth/signup` | `{username, email, password}` | user |
| POST | `/auth/login` | `{email, password}` | user + session cookie |
| POST | `/auth/logout` | | |
| GET | `/auth/me` | | current user |
| GET | `/world` | | your worlds |
| POST | `/world` | `{name, description}` | world |
| GET / PATCH / DELETE | `/world/{world_id}` | PATCH: `{new_name?, new_description?}` | world |
| GET | `/world/{world_id}/conversations` | | `{conversations: [...]}` |
| POST | `/world/{world_id}/conversations` | `{title}` | conversation |
| GET / DELETE | `/conversation/{id}` | | conversation with messages |
| POST | `/conversation/{id}/messages` | `{content}` (1–10,000 chars) | **SSE stream** (see below) |

Passwords need 8+ characters, upper and lower case, a number and a symbol.

### The chat stream

`POST /conversation/{id}/messages` answers with `text/event-stream`. Each `data:` line is one JSON event from Rig's agent stream. The useful ones:

| `type` | Meaning |
|---|---|
| `streamAssistantItem` with `value.event == "text"` | a piece of the reply, in `value.text` |
| `streamAssistantItem` with `value.event == "reasoning"` | a piece of the model's reasoning |
| `toolCall` / `toolResult` | the agent called `search_world` and got lore back |
| `finalResponse` | the run finished; token usage is in `usage` |

If the AI fails partway, the stream sends an `event: error` with a message and stops. Messages are saved only when the run finishes, so a failed turn leaves no half-saved history and can be retried.

**Heads-up:** these are Rig's own event shapes, passed through unchanged. They can change when Rig is upgraded.

## Tests

```bash
cargo test
```

Unit tests cover the pure logic with no database or AI involved: merging new info into saved entities ([extraction/tests.rs](src/service/ai/extraction/tests.rs)), plus RRF ranking and lore formatting ([retrieval/tests.rs](src/service/ai/retrieval/tests.rs)).

sqlx checks queries against the database at compile time, so `app-db` must be running for `cargo build` and `cargo test`.

To test the whole system by hand, use the [playground](playground/README.md).

## Logging

The app logs to stdout with `tracing`. The default level is `info`. Change it with `RUST_LOG`:

```bash
RUST_LOG="info,tower_http=debug" cargo run
RUST_LOG="lorekeeper_backend=debug" cargo run
```

Background extraction logs errors as `extraction failed for conversation <id>: ...`. Rig logs a `WARN` when an extraction attempt fails to parse and retries.

## Troubleshooting

| Symptom | Cause and fix |
|---|---|
| Build fails with `password authentication failed for user "dev"` | sqlx reached the wrong Postgres. Lorekeeper's is on **5433**. Check `docker ps` and `DATABASE_URL`. |
| Startup logs `Tried to retrieve an unknown database "neo4j"` | neo4rs must be told Memgraph's database name. `create_graph` sets `.db("memgraph")`, so keep it. |
| Startup panics `Invalid environment configuration.: MissingValue(...)` | A variable is missing from your environment. Export `.env` first (step 5). |
| Startup panics `Invalid secret key. Must be at least 64 bytes.` | `SESSION_SECRET_KEY` is too short. |
| rust-analyzer shows no compiler errors | Check `rust-analyzer.checkOnSave` isn't `false` in your editor settings. |
