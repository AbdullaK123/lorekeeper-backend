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

Five questions run again in separate empty conversations against the saved synthetic Ardent Reach world.
All five reviewed answers pass on 2026-10-30.
Chat calls `get_conflicts` and presents both saved contradictions without resolving them.
The other checks retain minor facts, relationship conditions, shared titles, and unknown facts.
The graph remains unchanged. All five background research runs complete.

See the [rerun evidence](../tests/artifacts/fresh-retrieval-2026-10-30/summary.md).
The [previous failure](../tests/artifacts/fresh-retrieval-2026-10-29/summary.md) remains recorded for comparison.

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
| 2026-10-30 | Reviewed GetConflicts draft: world scope is present; output, description, object arguments, unresolved filtering, and chat wiring remain unfinished |
| 2026-10-30 | Updated chat-tool tests; 25 native live checks pass; all five fresh-conversation answers pass, including both conflicts |

## Follow-up: conflict lookup verification

On 2026-10-30, completed the `get_conflicts` tool description.
It explains when to query conflicts, how to present disputed claims, and what an empty result means.
The draft now returns typed conflicts and filters unresolved records within the world.
Chat wiring is reviewed on 2026-10-30: app supplies the repository, and chat registers the scoped read-only tool.
The chat-tool test now verifies both read-only tools and the current factory signature.
Two new live tests verify unresolved filtering, world scope, sources, and empty object arguments.
All 25 native live database and worker checks pass.
The API suite reports 17 passes, two failed methods, and five skipped paid tests.
Duplicate signup returned 500 in that run; the fix below now passes its regression test.
Blank-name rejection remains an unconfirmed product rule.
Pure Rust tests report 11 passes and the same synthetic ranking failure.
The separate five-case paid smoke rerun passes all reviewed answer checks.

## Work selection rule

Prioritize observed failures and required product behavior.
Do not schedule speculative edge-case fixes without evidence that the application encounters them.
Label defensive tests separately from demonstrated runtime defects.

## Review outcome

**Goal result:** Accepted. Researcher extraction is complete for this sprint.
**Evidence:** Linked smoke transcripts, graph snapshots, source audits, and database tests.
**Carried work:** None automatically. Remaining work stays in the project plan.
**Next sprint candidate:** Choose the next observed defect or product story. Conflict retrieval now passes its regression check.

Choose the next sprint scope before replacing this document.
Project completion still requires all 15 epics and 69 stories.

### Duplicate signup trace — 2026-10-30

Reviewed the error path without changing runtime code.
The user insert passes the database error through `?`.
The infrastructure-to-service conversion treats every error as internal.
The HTTP mapping therefore returns 500 for an existing email.
A duplicate-account error mapped to 409 remains unfinished. No tests run for this trace.

### Duplicate signup fix — 2026-10-30

Signup now maps the users_email_key unique violation to a conflict error.
Infrastructure, service, and HTTP conversions preserve that error. The response is 409 with a clear message.
Other database errors keep their existing mapping. The database constraint remains in place.
The real duplicate-signup and login privacy regression tests both pass against the isolated QA database.
No model calls run. The full suite is not repeated for this change.
Blank-name rules and the synthetic ranking check remain separate work.

### Shared text validation reference — 2026-10-30

Reviewed Nexus Writer's `src/shared/text_types.py` and text-boundary tests.
Its rules cover length, NUL rejection, Unicode normalization, controls, whitespace, and field-specific text policies.
It separates single-line labels, multiline prose, and unchanged credentials.
It does not detect malicious intent or replace query parameters and output escaping.
Proposed starting scope: a validated single-line type for world names, with a 255-character limit.
No SafeText implementation or field changes exist yet. Reference tests are inspected, not executed.

### Shared text types — 2026-10-30

Added `src/shared/text.rs` with private String wrappers and fallible parsers.
Types: Username, WorldName, ConversationTitle, EntityName, SearchQuery, Description, ChatContent, and PasswordInput.
Names normalize Unicode and whitespace and reject direction controls, NUL, and blank values.
Prose preserves line breaks and language joiners. Descriptions allow empty text.
Passwords retain exact input and omit Debug and Display. Password strength remains separate.
All types enforce raw character limits; JSON decoding calls the same parser.
Added unicode-normalization using the cached locked version.
Seven focused tests pass, including JSON validation, Unicode, limits, and exact password preservation.
Existing request fields still use String. Wiring and HTTP regression tests remain unfinished.
These types do not detect malicious intent, sanitize HTML, or replace query parameters.

### Request-model text validation — 2026-10-30

World create/update requests now use WorldName and Description.
Signup uses Username and PasswordInput; login uses PasswordInput.
Conversation creation uses ConversationTitle; chat requests use ChatContent.
JSON parsing validates and normalizes these values before service calls.
Existing database rows and response fields remain Strings. Services unwrap validated input at storage boundaries.
Email validation and signup password-strength checks remain in place.
Login accepts bounded nonempty passwords without applying the signup strength policy.
Password Debug output is redacted so request Debug cannot expose credentials.
EntityName and SearchQuery are wired into AI tool arguments in the follow-up below.

Verification: all 19 existing non-AI API checks pass; five paid checks are skipped.
Four added HTTP checks pass: normalized storage, rejected updates, blank titles, and blank chat.
All eight focused text tests pass. No model calls run.
Blank world names now return 422, resolving the earlier validation failure.
The defensive ranking test remains outside this change; the full native suite is not repeated.

### AI tool text validation — 2026-10-30

SearchWorldArgs now uses SearchQuery.
Graph entity names and aliases use EntityName; entity and relationship descriptions use Description.
JSON tool arguments validate before embedding, retrieval, or graph writes.
Shared text schemas publish string types and minimum/maximum character counts.
Existing graph-ID checks and trusted world/source context remain unchanged.
Graph conversions unwrap normalized values and derive name keys from the normalized name.
Properties, relationship kinds, and IDs retain their existing validation rules.
Four focused tests cover malformed arguments, normalized conversion, and schema bounds.
Verification: four tool-argument tests, eight text tests, and 13 live graph/wiring checks pass.
No model calls run. The full API suite is not repeated for this change.

### Research job timing audit — 2026-10-30

Read retained log timestamps for all 50 research runs from the three worldbuilding smoke tests.
Researcher-call median: 23.3 seconds; mean: 24.6 seconds; range: 3.0–59.6 seconds.
Timing excludes preceding chat and job setup. Existing report seconds include chat plus research.
No new tests or model calls run. Queue delay and concurrent load remain unmeasured.
See [timing evidence](../tests/artifacts/research-timings-2026-10-29.md).

### Durable queue crate survey — 2026-10-30

Checked primary documentation for Apalis, Underway, and Fang.
Apalis supports durable workers with Postgres and Redis backends.
Underway supports Postgres jobs, retries, and enqueueing inside an application transaction.
Fang provides database-backed background processing.
Postgres-backed queues fit the existing infrastructure; no crate is selected or installed.
Next selection check: transaction integration and compatibility with the installed SQLx version.
No tests or model calls run for this survey.

### Rig durable-run investigation — 2026-10-30

Checked the installed Rig 0.44.0 source and official durable-run documentation.
Rig includes serializable AgentRun state and agent.resume(run).
An application can persist state between model/tool steps and reload it after restart.
This supplies agent continuation, not an automatic database queue or worker recovery service.
The application still owns checkpoint storage, scheduling, and recovery coordination.
Existing researcher.prompt calls do not save checkpoints.
No implementation change, tests, or model calls run. Queue selection remains open.
Reference: https://rig.rs/docs/concepts/runs

### Whole-job queue recommendation — 2026-10-30

Recommend evaluating Underway 0.2.0 for a single research task with whole-run retries.
Verified documented transactional enqueue, retry policy, timeout, heartbeat, and concurrency-key APIs.
Its SQLx dependency is ^0.8.2, compatible in version range with this project's 0.8.6.
No dependency resolution or runtime recovery test is performed yet. No crate is installed.
Graph replay safety remains application work; queue retries do not provide exactly-once Memgraph writes.
Sources: https://docs.rs/underway/0.2.0/underway/queue/struct.Queue.html and https://docs.rs/crate/underway/0.2.0

### Underway idempotency API check — 2026-10-30

Underway 0.2.0 documents no dedicated enqueue idempotency-key parameter.
Its concurrency_key serializes execution for matching keys; it does not deduplicate enqueue requests.
Application-level deduplication can use a unique source-message record in the enqueue transaction.
That prevents duplicate scheduling, not repeated Memgraph effects during job retries.
Documentation review only; no runtime checks or implementation changes.
