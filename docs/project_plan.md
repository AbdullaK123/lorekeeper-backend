# Lorekeeper Backend — Project Plan

## <span style="background-color:var(--color-highlight-bg-red)">~~Current State~~</span>

<span style="background-color:var(--color-highlight-bg-red)">~~**Done:** Auth end-to-end, all four repositories (user, world, conversation, graph), world service, error architecture, middleware stack, Docker Compose with Postgres + Memgraph. Solid foundation.~~</span>

<span style="background-color:var(--color-highlight-bg-red)">~~**Missing:** Everything that makes it a worldbuilding app — the AI pipeline, the chat flow, and the routes that expose worlds/conversations/graph to a client.~~</span>

<span style="background-color:var(--color-highlight-bg-red)">~~**Bug:** `add_convos` migration (timestamp `165747`) runs before `add_worlds` (`205312`) but references `worlds(id)`. Fresh DB will fail. Rename the convos migration to a later timestamp.~~</span>

---

## Phase 1: Wire What's Already Built

Get the existing repos exposed through the API. 1–2 sessions.

### <span style="background-color:var(--color-highlight-bg-red)">~~1. Fix migration ordering~~</span>
~~<span style="background-color:var(--color-highlight-bg-red)">Rename `add_convos` migration timestamp to sort after `add_worlds`.</span>~~ 
<span style="background-color:var(--color-highlight-bg-red)">~~**Artifact:** Migration file renamed, `sqlx migrate run` succeeds on fresh DB.~~</span>

### <span style="background-color:var(--color-highlight-bg-red)">~~2. Create ConversationService~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~Wraps `ConversationRepository`. Create, list by world, get by id, append message, delete. Same pattern as `WorldService`.~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~**Artifact:** `src/service/conversation.rs`~~</span>

### <span style="background-color:var(--color-highlight-bg-red)">~~3. World controller + routes~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~CRUD endpoints:~~</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~POST /worlds~~`</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~GET /worlds~~`</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~GET /worlds/:id~~`</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~PATCH /worlds/:id~~`</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~DELETE /worlds/:id~~`</span>

<span style="background-color:var(--color-highlight-bg-red)">~~Add `WorldService` to `AppState`.~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~**Artifact:** `src/app/controllers/world.rs`, routes wired in `app.rs`~~</span>

### <span style="background-color:var(--color-highlight-bg-red)">~~4. Conversation controller + routes~~</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~POST /worlds/:id/conversations~~`</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~GET /worlds/:id/conversations~~`</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~GET /conversations/:id~~`</span>
- <span style="background-color:var(--color-highlight-bg-red)">`~~DELETE /conversations/:id~~`</span>

<span style="background-color:var(--color-highlight-bg-red)">~~**Artifact:** `src/app/controllers/conversation.rs`, routes wired in `app.rs`~~</span>

<span style="background-color:var(--color-highlight-bg-red)">~~**Milestone:** Working API where users can sign up, create worlds, and create conversations. No AI yet, but the shell is demoable.~~</span>

---

## Phase 2: AI Integration — The Core

This is the critical path. The whole app is this. 3–5 sessions.

### <span style="background-color:var(--color-highlight-bg-red)">~~5. Add Rig to dependencies~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~`rig-core` in `Cargo.toml`. Verify it compiles with Rust edition 2024.~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~**Artifact:** `Cargo.toml` updated, `cargo check` passes~~</span>

### <span style="background-color:var(--color-highlight-bg-red)">~~6. Write the extraction prompts~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~`prompts/` directory with the system prompt for lore extraction. What the agent should extract, how it should reason about entities vs relationships vs conflicts, when to update vs create. This is the extraction skill in markdown.~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~**Artifact:** `prompts/extraction.md`~~</span>

### <span style="background-color:var(--color-highlight-bg-red)">~~7. Define extraction types~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~Rig's `Extractor<T>` needs target structs. Define the structured output the LLM returns: extracted entities, relationships, conflicts. These map to the graph models.~~</span>
<span style="background-color:var(--color-highlight-bg-red)">~~**Artifact:** `src/service/ai/extraction.rs`~~</span>

### 8. Implement entity tool
Rig `Tool` trait. The agent calls this to create/update/get entities in the graph. Wraps `GraphRepository` entity methods.
**Artifact:** `src/service/ai/tools/entities.rs`

### 9. Implement relationship tool
Same pattern, wraps graph relationship methods.
**Artifact:** `src/service/ai/tools/relationships.rs`

### 10. Implement conflict tool
Wraps graph conflict methods. When extraction finds a contradiction with existing lore, it creates a conflict node.
**Artifact:** `src/service/ai/tools/conflicts.rs`

### 11. Implement conversation tool
Wraps conversation repo for the agent to read conversation history, update summaries.
**Artifact:** `src/service/ai/tools/conversations.rs`

### 12. Build the agent
The orchestrator. Takes a user message, loads conversation history, runs the Rig agent with the tools, extracts lore, writes to graph, returns the response. This is the vertical slice that ties everything together.
**Artifact:** `src/service/ai/agent.rs` — agent that takes a message and returns a response with side-effects in the graph

**Risk:** This is where Rig, the tools, the prompts, and the graph all have to work together. Time-box the first attempt at two hours. If Rig's agent loop doesn't play well with the async Axum setup, you'll know fast and can adapt.

**Milestone:** AI agent can receive a message, respond conversationally, and write extracted lore to the knowledge graph.

---

## Phase 3: Chat Endpoint

Connect the agent to the API. 1 session.

### 13. Chat endpoint
`POST /conversations/:id/messages`. Takes a user message, runs it through the agent, appends both user and assistant messages to the conversation, returns the response. Start with plain REST (request-response).
**Artifact:** Handler in conversation controller, wired

**Milestone: V1.** A user can sign up, create a world, start a conversation, send messages, and the AI extracts lore into the knowledge graph behind the scenes.

---

## Phase 4: Graph Visibility

Users need to see what the AI extracted. 1 session.

### 14. Graph query endpoints
- `GET /worlds/:id/entities`
- `GET /entities/:id/neighbors`
- `GET /worlds/:id/relationships`
- `GET /worlds/:id/conflicts`

**Artifact:** `src/app/controllers/graph.rs`, routes wired

---

## V2 — Cut From V1

- WebSocket streaming for chat responses
- Conflict resolution UI (user approves/rejects detected contradictions)
- Graph visualization endpoint (full subgraph export for frontend renderer)
- Conversation summarization (auto-title, periodic summary for long conversations)
- Entity search / fuzzy matching
- Prompt refinement based on real usage patterns

---

## Critical Path

```
1 → 2 → 3 → 4 → 5 → 6 → 7 → 8–11 (any order) → 12 → 13
```

Phase 4 is independent once the graph repo is already done.

## Sequencing Notes

- Tasks 8–11 (the four tools) have no dependencies on each other. Do them in whatever order feels natural — entities first is probably easiest since it's the most straightforward CRUD.
- Task 6 (prompts) and task 7 (extraction types) can be done in either order, but thinking about the prompts first usually produces better extraction types.
- Phase 1 is pure wiring — no new logic, just exposing what exists. Fast wins, visible progress.
- Phase 2 is where the real work lives. Don't rush it.