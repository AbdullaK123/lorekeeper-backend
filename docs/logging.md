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
Original research jobs use the source message ID as the workflow ID.
A forked retry has another workflow ID. The retry submission event records that active ID.
The research_workflow registration span still labels its source message ID as workflow_id; it is not the fork ID.
Use the source message ID plus the retry mapping to connect attempts.
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


## Retry diagnostics — 2026-11-02

The conversation.retry_research span includes world, user, and source message IDs.
A successful retry submission logs its active workflow ID.
Status reads follow the saved current-attempt mapping.
A submission error can occur after chat storage; the source message remains readable.
Logging and capture tests do not prove cross-database commit atomicity.

## Auth email spans — 2026-11-07

Auth handlers, password hashing, reset and verification services, email sending,
and token/user writes now have named operation spans.
All new spans use skip_all. Service and repository spans add only user_id where available.
They do not record token values, hashes, passwords, email addresses, URLs, keys, or return values.

## Auth outcome events — 2026-11-07

Reset and verification log rejected tokens, rollback attempts, completed rollbacks, and committed changes.
Transaction failures identify begin, commit, or rollback without logging raw errors.
Email logs separate template failures, provider submission failures, and provider acceptance.
Provider acceptance does not mean delivery to the recipient.
Token consume logs report a matching row before commit; the caller still owns the transaction.
Email request rejections record missing users or already-verified accounts without recipient addresses.

## Custom auth sessions — 2026-11-08

The UserId extractor, auth session methods, parent repository methods, and session child queries have named spans.
Spans skip all arguments and return values. They include explicit user IDs where available.
Events record missing cookies, missing/expired sessions, resolved user IDs, expiry, and deletion counts.
Deletion counts inside repository queries describe executed statements. Password-reset commit events establish persistence.
Raw cookie tokens, token hashes, and cookie headers are never included in these events.

## Google login start — 2026-11-10

Google client setup, the start handler/service, and login-attempt child operations have named spans using skip_all.
Setup failures record only the provider and failing stage. Database failures record only the operation.
Events distinguish provider discovery, client readiness, saved attempts, prepared redirects, and matched/missing consume results.
They never include settings, client secrets, state values/hashes, nonce, PKCE verifier, generated URLs, or return values.
A prepared redirect is not a completed Google sign-in. Callback handling remains unimplemented.
