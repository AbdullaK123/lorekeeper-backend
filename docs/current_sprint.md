# Current Sprint

## Sprint 1: Replace Graph Extraction with the Researcher

**Status:** Complete for the agreed pipeline scope.
**Start:** 2026-10-27.
**Planned review:** 2026-11-03.
**Reviewed:** 2026-10-29.
**Planned length:** One week; reviewed early after completing the expanded scope.

This document tracks the current work cycle and its outcome.
Review and replace it every one to four weeks, or when a sprint finishes early.
Archive it under `docs/sprints/` before starting the next sprint.

## Goal and scope change

Replace brittle extraction and name matching with a tool-using background researcher.
The researcher interprets language. Code enforces exact IDs, world scope, transactional writes, and message-source storage.

The sprint starts as read-tool design work.
The writer expands it to include graph writes, runtime replacement, source tracking, and live smoke tests.
The writer accepts the extraction pipeline as done for now on 2026-10-29.

This work supports Epics 3, 4, and 5 in the [user stories](user_stories.md).
It advances automatic saving, contextual identity, preserved facts, contradictions, and traceable sources.
It does not complete these epics or their client workflows.

## Completed work

| Task | State | Evidence |
| --- | --- | --- |
| Write the researcher prompt using the seven-section template | Done | [Researcher prompt](../prompts/researcher.md) |
| Reuse world-scoped search and expose saved IDs and aliases | Done | [Search tool](../src/service/ai/tools/entities/search.rs) |
| Read complete entities, relationships, properties, conditions, conflicts, and sources | Done | [Full-read tool](../src/service/ai/tools/entities/read.rs) |
| Save graph batches using exact IDs and one transaction per call | Done | [Graph tool](../src/service/ai/tools/entities/graph.rs), [repository](../src/data/repositories/graph.rs) |
| Generate new graph IDs in code | Done | New records omit IDs; saved results return server-generated IDs |
| Keep chat read-only and start research after successful message storage | Done | [Conversation service](../src/service/conversation.rs), [researcher](../src/service/ai/researcher.rs) |
| Record contradictions without replacing established canon | Done | [Conflict tool](../src/service/ai/tools/conflicts.rs) |
| Give user, assistant, and tool messages stable IDs | Done | [Message models](../src/data/models/conversations.rs), [backfill migration](../migrations/20261029000000_add_message_ids.sql) |
| Store trusted message sources on entities, relationships, and conflicts | Done | Source lists accumulate without duplicate IDs; older records remain readable |
| Remove the old extraction pipeline and obsolete types, prompts, and write methods | Done | Runtime uses the researcher only for AI lore writes |
| Move dense conversions into traits and named methods | Done | Typed graph results, history conversion, list conversion, and formatting helpers |
| Verify realistic conversations and source preservation | Done | Smoke runs and database checks below |

## Acceptance evidence

| Check | Result |
| --- | --- |
| World isolation and exact-ID writes | Live graph tests pass |
| Atomic batch failure | Failed relationship writes roll back preceding batch changes |
| Preserved details | Smoke runs retain minor facts through later updates |
| Contextual identity | Captain and Warden clarifications affect the intended subjects |
| Explicit corrections | Birthplace and route changes preserve unrelated lore |
| Relationship conditions | Graph records preserve seasonal and historical qualifiers |
| Unresolved contradictions | Conflicts remain recorded without invented repairs or exceptions |
| Hypothetical isolation | Tested hypothetical turns change neither graph facts nor source lists |
| Stable message identity | Backfill, repeated database reads, and concurrent append tests pass |
| Source tracking | Smoke sources resolve to saved user messages; earlier sources survive updates |
| Conflict sources | New conflicts point to the exact reporting messages; retry deduplication and legacy reads pass |

### Smoke runs

- [First 20-turn run](../tests/artifacts/smoke-2026-10-29/summary.md): pipeline behavior before message-source tracking.
- [Second 20-turn run](../tests/artifacts/smoke-2026-10-29-run2/summary.md): entity and relationship message sources.
- [New 10-turn world](../tests/artifacts/smoke-2026-10-29-10turns/summary.md): conflict message sources, corrections, ambiguity, and hypothetical isolation.

All three runs complete without reported chat or research failures.
The latest graph regression run passes 16 tests.
See the [test report](../tests/REPORT.md) for scope and remaining failures.

## Remaining work outside the completed pipeline scope

- Durable research jobs, restart recovery, and visible retry status.
- Concurrent research updates and coordination with world deletion.
- Support for distinct subjects sharing an identical canonical name; the unique-name constraint remains.
- Explicit search truncation reporting and broader ambiguity evaluations.
- Source navigation in answers and the client.
- Source behavior after conversation deletion and claim-level source attribution.
- Separate original-claim and disputed-claim sources for conflicts.
- Removal of redundant model-written source metadata inside arbitrary properties.
- Graph exploration, conflict review, and the remaining product and SaaS epics.

The smoke runs are sequential and retain full conversation history.
They do not prove concurrent safety, recovery, or retrieval from a fresh conversation.
These limits do not reopen the accepted extraction pipeline scope.
They remain tracked work in the [project plan](project_plan.md) and [lifecycle design](design.md).

## Retrieval follow-up

Follow-up formatting work resolves both observed context-formatting failures:

- Answer context now includes neighbor properties.
- Answer context now includes relationship descriptions and properties, preserving stored conditions.
- Duplicate-row ranking is a defensive case. Current queries have not demonstrated duplicate results.

The latest pure Rust test run reports 11 passes and one failing synthetic duplicate-row test.
That failure does not establish a runtime defect and is not scheduled for a fix.
See the [test report](../tests/REPORT.md) for verification details.

## Fresh-conversation verification

Five questions run in separate empty conversations against the saved Ardent Reach world.
Four pass: minor details and corrections, relationship conditions, shared titles, and unknown facts.
One fails: chat reports no unresolved contradictions despite two saved unresolved conflicts.
The graph remains unchanged throughout the checks.

Search returns entities and neighboring entities, excluding conflict nodes.
The chat agent has no other path to read saved conflicts.
This is a demonstrated defect and the next retrieval task.

See the [answers and code trace](../tests/artifacts/fresh-retrieval-2026-10-29/summary.md).

## Sprint log

| Date | Change or result |
| --- | --- |
| 2026-10-27 | Sprint starts with researcher read-tool contracts |
| 2026-10-29 | Writer expands scope to implement the researcher and remove the old extraction pipeline |
| 2026-10-29 | Read-only chat and background researcher wiring verified |
| 2026-10-29 | Stable message IDs and source tracking added across lore and conflicts |
| 2026-10-29 | Two 20-turn runs and one new 10-turn world inspected in Memgraph |
| 2026-10-29 | Writer accepts the extraction pipeline as done for now |
| 2026-10-29 | Follow-up formatting fixes preserve neighbor and relationship details; 11 pure tests pass, one synthetic ranking test fails |
| 2026-10-29 | Repository AGENTS.md requires a sprint update after completed work |
| 2026-10-29 | Fresh-conversation checks: four pass; saved conflicts fail to reach chat; graph remains unchanged |

## Work selection rule

Prioritize observed failures and required product behavior.
Do not schedule speculative edge-case fixes without evidence that the application encounters them.
Label defensive tests separately from demonstrated runtime defects.

## Review outcome

**Goal result:** Accepted. Researcher extraction is complete for this sprint.
**Evidence:** Linked smoke transcripts, graph snapshots, source audits, and database tests.
**Carried work:** None automatically. Remaining work stays in the project plan.
**Next sprint candidate:** Expose saved unresolved conflicts to read-only chat retrieval, then repeat the failing fresh-conversation check.

Choose the next sprint scope before replacing this document.
Project completion still requires all 15 epics and 69 stories.
