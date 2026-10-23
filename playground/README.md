# Lorekeeper Playground

A small React app for testing the backend by hand. It's a dev tool, not the product UI.

Use it to chat with a world the way a writer would, then check what the backend did: which tools the AI called, what lore the search returned, and how long the reply took.

## What it does

- **Log in or sign up** against the real backend.
- **Create worlds and conversations** in the left sidebar.
- **Chat** in the middle. Replies stream in word by word, and **Stop** cancels a reply.
- **Inspect** the last reply in the right panel:
  - **Trace:** time to first word, total time, tokens, cost, each `search_world` call with its query and result, and the model's reasoning.
  - **Raw:** every SSE event exactly as the backend sent it.

## Run it

The backend must be running on port 8000 (see the [main README](../README.md)).

```bash
cd playground
npm install      # first time only
npm run dev
```

Open http://localhost:5173.

## How it talks to the backend

The Vite dev server forwards every `/api/*` request to `http://localhost:8000` and strips the `/api` prefix (see [vite.config.js](vite.config.js)). The browser sees one origin, so the session cookie works with no CORS setup.

`EventSource`, the browser's built-in SSE client, only supports `GET`. Chat turns are `POST`, so [src/api.js](src/api.js) reads the SSE stream by hand with `fetch`.

## Check what extraction saved

Extraction runs in the background a few seconds after each reply. To see the graph, query Memgraph directly:

```bash
docker exec -it lorekeeper-backend-app-memgraph-1 mgconsole
```

```cypher
// all entities in a world
MATCH (e:Entity {world_id: "<world id>"}) RETURN e.name, e.kind, e.description;

// all connections
MATCH (a:Entity {world_id: "<world id>"})-[r]->(b) RETURN a.name, type(r), b.name;

// open conflicts
MATCH (e:Entity {world_id: "<world id>"})-[:HAS_CONFLICT]->(c:Conflict) RETURN e.name, c.field, c.explanation;
```

To find a world's ID, open http://localhost:5173/api/world while logged in. It lists your worlds with their IDs.

## Known limits

- **It reads Rig's raw event format.** The backend forwards Rig's stream events as they are. A Rig upgrade can change that format and break the chat view here. The parsing lives in `applyEvent` in [src/App.jsx](src/App.jsx).
- **No graph view.** The backend has no graph endpoints yet, so use `mgconsole` as shown above.
- **Desktop only.** The inspector hides below 1100px, and the sidebar below 720px.
