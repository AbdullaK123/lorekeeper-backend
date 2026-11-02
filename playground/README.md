# Lorekeeper Playground

Updated: 2026-11-02.

A development UI for testing the backend contracts used by the product UI.
It is a testing tool, not the finished writer-facing product.
It uses real sessions, worlds, conversations, and backend responses.

## Run

Start the backend on port 8000 using the [root setup](../README.md#local-setup).

```sh
cd playground
npm install
npm run dev
```

Open http://localhost:5173.
Vite forwards /api requests to the backend and preserves session cookies.

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
