# Lorekeeper Playground

Updated: 2026-11-02.

A development UI for testing the backend contracts used by the product UI.
It is a testing tool, not the finished writer-facing product.
It uses real sessions, worlds, conversations, and backend responses.

## Run as static files from the backend

Build the playground once:

```sh
cd playground
npm install
npm run build
```

Start the backend with ENV=dev using the [root setup](../README.md#local-setup).
Open http://localhost:8000/. Axum serves only the built files in playground/dist.
The dev-only /api alias forwards the existing client requests to the backend routes with session cookies.
Rebuild after frontend changes. No Node or Vite server runs while Axum serves these files.
The source files and node_modules are not served.
ENV=staging and ENV=prod expose neither playground files nor the /api alias.
Normal API routes remain available in every environment.

## Container build

The root Dockerfile builds the playground in a Node stage and copies only dist into the API runtime image.
It also builds the API in a separate Rust stage. No Node server runs in the final image.
With ENV=dev, open the API's published port at http://localhost:8000/.
The same image keeps static serving disabled in staging and production.
See [container setup](../README.md#container-build) for build and migration commands.

## Run with frontend live reload

For frontend editing, keep the backend on port 8000 and run:

```sh
cd playground
npm run dev
```

Open http://localhost:5173. Vite proxies /api requests to the backend's normal routes.
The built static copy and live-reload copy use the same client paths.

## Google sign-in

Select **Continue with Google** on the login or signup screen.
Allow the sign-in window. The playground checks `/auth/me` once per second.
After login, it closes the window and loads your account.
If Google reports an error, close the window and retry.
The check stops after ten minutes, on session-check errors, or when you close the window.
The backend still returns JSON from its callback. No callback redirect changes are needed.
Use the same hostname for the playground and `API_BASE_URL`, so cookies reach both.
The backend must run the current Google routes and have working Google credentials.

Browser regression test:

```sh
PLAYWRIGHT_MODULE=/path/to/playwright/index.mjs \
PLAYWRIGHT_CHROMIUM_EXECUTABLE=/path/to/chromium \
QA_BASE_URL=http://127.0.0.1:5173 node tests/playwright_oauth.mjs
```

This test uses controlled OAuth/session responses. It checks the UI, not Google's live service.

## Chat and stream

1. Log in or create a test account.
2. Create or select a world, then a conversation.
3. Send a message and inspect the streamed reply.
4. Use Stop to abort the browser's stream request.

The inspector shows stream events, model text, tools, timing, and reported usage.
Saved chat bubbles display message IDs for source and research checks.
Conversation requests use /world/{world_id}/conversations/{conversation_id}.
Changing the world or conversation aborts the active browser stream and ignores late results from that selection.
Stopping the browser stream does not guarantee cancellation of backend research.

## API workbench

Select a world, then open API workbench. A conversation is optional for graph reads.

| Request | Controls |
| --- | --- |
| Graph and entity pages | Limit, cursor, and entity-kind filter where supported |
| Entity search | Text query and limit |
| Entity details and connections | Entity ID, edge pagination, kind/type filters |
| Relationship details | Relationship ID |
| Conflict list/details | Resolved filter, pagination, conflict ID |
| Conflict decisions | Keep existing or accept new |
| Source message | Message ID or saved-message selector |
| Research status and retry | Message ID or saved-message selector |
| Saved conversation and deletion | Conversation ID, defaulting to the selected conversation |

The workbench shows the exact request path, HTTP status, duration, and JSON response.
HTTP errors remain visible as responses. Network errors appear separately.
Use next cursor copies the server's cursor into the form; send another request to load that page.
Empty pages remain visible as JSON. No polling or mutation starts automatically.

Conflict decisions change stored canon where supported. Conversation deletion removes saved history.
An accepted retry can make paid model and embedding calls.
See the [retry contract](../docs/research_retry.md) for eligibility and recovery limits.

## Suggested test flow

1. Chat about a synthetic world, then copy the saved user message ID.
2. Check Research status until it completes or fails.
3. Inspect entity search and graph pages after completion.
4. Open entity, relationship, conflict, and source IDs from the returned JSON.
5. Resolve a supported conflict and read it back.
6. Test retry on an eligible failed or not-submitted source; inspect the returned status.
7. Inspect raw errors for invalid IDs, filters, and ineligible retries.

Graph exploration currently uses JSON rather than an interactive graph layout.
Claim-level citations, the full product conflict/recovery flow, world editing, and billing screens remain outside this tool.
The stream inspector hides below 1100px. World/conversation selection remains available on small screens.

## Verify changes

```sh
npm test
npm run build
```

Six client tests cover request encoding, filters/cursors, missing IDs, conflict/retry request bodies,
HTTP error inspection, session credentials, and nested conversation URLs.
They replace fetch locally and make no backend or model calls.
A server-render check verifies the workbench's initial controls; it is not a browser interaction test.
The production build checks JSX and module integration.

Use [backend tests](../tests/README.md) for real HTTP contracts and database behavior.
Rig's SSE event shapes can change when Rig changes; the inspector parser may need updates.


## Browser verification — 2026-11-03

A real Playwright run covers signup, world/conversation creation, chat, research polling,
graph/source inspection, conflict acceptance, and visible HTTP errors.
The UI flow exposes a backend canon-preservation failure: accepted age 41 later becomes 34 again.
See [browser evidence](../tests/artifacts/playwright-2026-11-03/summary.md) and [paid-run instructions](../tests/README.md#real-playwright-writer-workflow-paid).
This is failing acceptance evidence; it does not mark the product conflict workflow complete.


### Canon regression rerun

The [source-fix rerun](../tests/artifacts/playwright-canon-decisions-2026-11-03/summary.md) passes all 10 steps.
Same-thread and fresh-thread questions answer accepted age 41 without changing stored lore or source IDs.
The old description remains stale; this test verifies current property and writer-decision authority, not summary regeneration.
