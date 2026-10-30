# DBOS research integration — 2026-10-30

## Implemented behavior

Chat saves both messages, then starts the research workflow using the saved user message ID as workflow ID.
The registered workflow reconstructs the scoped researcher from ResearchInput.
The researcher drives Rig's AgentRun using checkpointed model calls and sequential checkpointed tool calls.
During recovery, recorded responses reconstruct the loop without re-executing their operations.
The driver preserves DBOS errors and typed tool output. Unknown tools fail explicitly.

## Verification

| Check | Result |
| --- | --- |
| All-target cargo check | Pass |
| Same workflow submitted twice | Two model calls and two tool executions total |
| Restart DBOS during final model request | Recorded initial response and both tool results replay; only final model request repeats |
| Unknown tool | Fails without executing a tool |
| Invalid tool arguments | Invalid body does not execute; model receives results and continues |
| Native live database/worker checks | 30 pass |
| API checks | 23 pass, five paid skips |
| Text tests | Eight pass |

The live target reports 31 passes because it includes the API wrapper.
The restart test shuts down and recreates the engine in one process.
All provider calls use a local scripted HTTP service. No paid models run.

Commands use DATABASE_URL pointing to lorekeeper_adversarial:

```bash
cargo check --offline --all-targets
cargo test --offline --test adversarial_live durable_research -- --ignored --test-threads=1
QA_AI=0 cargo test --offline --test adversarial_live -- --ignored --test-threads=1 --skip researcher_smoke_conversation
cargo test --offline --test text_types
```

Logs: /tmp/lorekeeper-durable-replay.log, /tmp/lorekeeper-durable-regressions.log, /tmp/lorekeeper-durable-text.log.

## Remaining limits

- A committed Memgraph write can repeat if its DBOS checkpoint was not saved before interruption.
- Chat message storage and workflow submission are separate commits.
- No OS process kill test or new paid conversation smoke test runs.
- Job-status UI and concurrent research/deletion coordination remain separate work.
