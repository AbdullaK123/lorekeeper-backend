# Application logging

The application uses structured `tracing` events and spans.
A span groups events from one operation and carries its IDs.

## Levels

| Level | Purpose |
| --- | --- |
| `info` | Startup, requests, auth outcomes, saved changes, workflow lifecycle, cleanup, and shutdown |
| `debug` | Repository operations, lookup counts, search results, model/tool timings, and research status reads |
| `warn` | Invalid model tool requests and transaction rollback attempts |
| `error` | Failed requests, chat streams, workflow submission/execution, and cleanup |

The default level is `info`. Enable detailed application logs with:

```sh
RUST_LOG=info,lorekeeper_backend=debug cargo run
```

This enables application detail without enabling debug output for every dependency.

## Following an operation

HTTP events contain the method, route template, status, and response latency.
Service spans carry available user, world, conversation, message, and record IDs.
Research events use the source message ID as the workflow ID.
Chat events distinguish message storage from workflow submission.
Model and tool events inside DBOS steps describe executed calls; checkpoint replay skips those calls.
Graph events distinguish committed batches from rollback attempts.
Cleanup events identify jobs retained for another attempt.

## Private data

Application spans skip function arguments unless an explicit ID field is added.
Logs omit passwords, hashes, sessions, request bodies, query strings, lore, prompts, and model/tool payloads.
HTTP spans use route templates, including for text-search routes.
Request failures log status and category instead of internal error text.
Library logging remains controlled by its own tracing targets and the chosen `RUST_LOG` filter.

## Verification

The capture tests in `tests/cases/logging.rs` verify request and error-log privacy.
Database, workflow replay, and selected HTTP regressions verify behavior after instrumentation.
