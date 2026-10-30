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

- Atomic chat/job handoff and visible retry status; DBOS checkpoint replay is verified below.
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

### Idempotency wrapper crate investigation — 2026-10-30

Checked primary docs for idempotent 2.0.0 and agent-effects 0.2.0.
Idempotent offers keyed claims and cached results; its documented expired-lease case can leave an effect applied without a cached result.
Agent-effects explicitly requires target-system cooperation for exactly-once effects and tracks unknown outcomes.
Neither can force arbitrary external Memgraph writes to become idempotent across crashes.
A stable operation key checked and recorded in the same graph transaction as its writes can protect that exact operation.
Replaying an LLM run still needs stable operation identity; deduplicating a job alone does not provide it.
Documentation review only. No crate installed or model calls run.
Sources: https://docs.rs/idempotent/latest/idempotent/ and https://docs.rs/agent-effects/latest/agent_effects/

### Underway setup starting point — 2026-10-30

Initial task: install Underway 0.2 and run its migrations after create_pool in app startup.
No worker, enqueue path, or retry behavior is implemented yet.
Agreed minimal direction: durable pending jobs, one research job at a time, no automatic replay of failed/interrupted work.
Dependency installation and migration execution remain unverified.

### Official DBOS Rust SDK check — 2026-10-30

Confirmed the official dbos-inc/dbos-transact-rust repository and dbos crate.
DBOS stores workflow step results in Postgres and supports restart recovery without a separate orchestration service.
The official README states Rust has no transactional steps yet and external writes are at-least-once.
DBOS is an alternative to Underway for evaluation; neither is installed or selected by this review.
A whole-researcher step would still repeat unfinished graph effects after a crash.
Source: https://github.com/dbos-inc/dbos-transact-rust
No tests or model calls run.

### DBOS Rust integration outline — 2026-10-30

Reviewed official Rust source for DBOS lifecycle, registration, step, and StepOptions.
Proposed integration points: register research at startup, launch DBOS, submit from chat, checkpoint research via step.
Rust uses explicit workflow registration and step closures; Python decorators do not apply.
Step max_attempts defaults to one, but shutdown/crash leaves an unfinished step recoverable and it can execute again.
Thus one whole-researcher step does not meet the earlier no-automatic-replay design without further policy.
SDK source labels the implementation under construction. No dependency or runtime code changes made.
Sources: official dbos-inc/dbos-transact-rust crates/dbos/src/{lib,instance,step}.rs.

### Nexus Writer research durability reference — 2026-10-30

Read live research_agent.py, research_agent_flow.py, and jobs.py in nexus-writer-backend.
The agent attaches DBOSDurability; all custom research tools are explicitly wrapped with DBOS.step.
Official Pydantic AI docs confirm the capability checkpoints model requests, while custom tools require explicit wrapping.
The outer workflow separately checkpoints report storage, embedding, notification, and lock release.
Submission uses SetWorkflowID(run_id). This is step-level agent replay, not one opaque researcher step.
Rust inspiration: checkpoint individual model and tool calls, using Rig's stepping API as the integration boundary.
No Rig/DBOS adapter is implemented or verified here. External writes retain the crash-before-checkpoint limit.
No model calls or tests run during this reference review.

### DBOS Rust startup lesson — 2026-10-30

Checked official Config, DBOS lifecycle, and starter source before giving setup guidance.
The project currently has no DBOS dependency. First task: add dbos, configure the database URL and application version, register a small workflow, then launch.
Launch applies system migrations by default; shutdown belongs after server termination.
Guidance follows current official source; resolved crate version and compilation remain unverified.
No runtime code or dependencies changed during this lesson.

### DBOS startup implementation review — 2026-10-30

User added dbos 0.5.0, setup_dbos, app startup wiring, and shutdown after Axum exits.
Review finds register_workflow maps errors but omits ?, so registration failure is ignored.
The echo WorkflowRef is local and not returned for invoking the setup exercise.
All-target cargo check fails at two live-test harness call sites: create_app now returns three values, while they expect two.
The application binary checks successfully with warnings. No launch or database migration is exercised.
Remaining: propagate registration failure, update test lifecycle to retain/shut down DBOS, then run echo.
No runtime code changed during review.

### DBOS test harness update — 2026-10-30

Both live API and paid smoke harnesses now capture the DBOS handle returned by create_app.
They shut down DBOS and abort the server before checking the scheduler shutdown result.
User's workflow registration now propagates errors with ?.
All-target cargo check passes with warnings.
The real non-AI API suite passes: 23 checks pass and five paid checks are skipped.
This exercises DBOS startup, system migrations, and shutdown against isolated QA services.
The paid smoke harness compiles but is not executed. Echo invocation and crash recovery remain untested.
No model calls run.

### Next DBOS exercise — 2026-10-30

Verified WorkflowRef::run(input) in the installed dbos 0.5.0 source.
Next exercise: invoke the registered echo workflow after launch, assert the returned text, and inspect its stored workflow outcome.
The invocation remains a user task; no code change or execution is performed for this guidance.

### Echo invocation review — 2026-10-30

User adds echo.run after launch, propagates errors, and logs its returned value.
Code review confirms the expected ordering; runtime execution is not independently verified in this turn.
Next exercise: wrap echo work in dbos::step so its result has a checkpoint.
Restart recovery testing follows after the step exists.

### Echo step live verification — 2026-10-30

Direct backend startup cannot bind port 8000 because another process already uses it; that process is untouched.
Added an opt-in isolated QA test for the existing setup_dbos path instead.
The test passes: workflow registration, launch, echo step execution, expected result, and shutdown complete.
Logs contain Executing echo step and Echo workflow completed with result=DBOS works.
DBOS reports launched before shutdown and not launched afterward.
No AI calls run. This proves basic execution, not crash recovery.
Evidence: /tmp/lorekeeper-dbos-echo-test.log. Next: a controlled recovery exercise.

### Researcher dependency handoff review — 2026-10-30

User passes a ResearcherService clone to ConversationService and moves the original into setup_dbos.
Ownership wiring is valid by inspection. DBOS setup accepts the service but does not use it yet.
Next: define a serializable research workflow input matching process_message arguments.
Echo registration and startup invocation remain temporary. Chat still starts research directly.
No compilation or runtime tests run for this review. The direct DBOS startup test needs the new service argument.

### App-layer DBOS move review — 2026-10-30

DBOS setup now lives in src/app/dbos.rs and app exports it; infrastructure no longer exports it.
ResearchInput exists with owned fields and Clone/Serialize/Deserialize.
The move is incomplete: app/app.rs still imports setup_dbos from infrastructure; the direct startup test also uses the old path/signature.
ResearchInput fields are private, so other modules cannot construct or read the input yet.
Review only; no compilation or runtime tests run.

### Research workflow preparation review — 2026-10-30

Confirmed app imports setup_dbos from the app layer and ResearchInput fields are public.
Next task is replacing echo registration with a research workflow accepting ResearchInput.
Installed DBOS registration requires a reusable Fn closure; clone the captured researcher per invocation before async move.
Remove echo's automatic startup invocation when replacing it. Researcher execution still needs per-model/tool checkpoints.
No build or execution performed. The outdated direct setup test remains pending.

### Research registration draft review — 2026-10-30

Research workflow now accepts ResearchInput and calls process_message; echo startup execution is removed.
Draft still clones the service outside the reusable closure, then moves it into the future.
Move cloning inside a move closure before creating the async future.
The body also discards its Result and returns unit; result/error conversion remains the next change.
No compile or runtime check performed; no registered research job has run.

### Research workflow result guidance — 2026-10-30

Confirmed the workflow now clones ResearcherService inside the reusable closure.
Next: return process_message's Result through DBOS's typed application-error variant.
Installed DBOS requires application errors to implement serde serialization/deserialization and std::error::Error.
ServiceError currently lacks serde derives; add them before mapping with dbos::Error::Application.
Guidance only; no compilation or model execution performed.

### Research result draft review — 2026-10-30

The workflow now maps ServiceError to DBOS Application errors, but retains a let binding without returning it.
ServiceError derives Serialize and Deserialize, but imports only Deserialize.
Next fixes: make the service call the async block's final expression and import both serde derives.
Compilation and workflow execution remain unverified. Research handle is not exposed and agent calls are not checkpointed yet.

### Research registration compile attempt — 2026-10-30

Confirmed the workflow returns the mapped researcher result and ServiceError imports both serde derives.
Cargo check is interrupted by missing target/debug build files during dependency compilation.
Errors include missing invoked.timestamp, object files, and rmeta temporary directories; disk has free space.
This does not verify compilation of the new application code. No workflow/model calls run.
Next integration task: return the typed research WorkflowRef alongside DBOS so callers can retain it.
Per-model/tool checkpoints and the outdated startup test remain unfinished.

### Research submission wiring guidance — 2026-10-30

ConversationService now has a typed research_job field but submission is unfinished.
Verified installed DBOS start_with persists and starts work without awaiting its result; workflow_id deduplicates submissions.
Guidance uses saved user-message ID as workflow ID and service.research_job inside the owned stream.
Remaining constructor wiring: remove obsolete direct researcher dependency, pass research handle, and import ServiceError in app/dbos.rs.
No code edited or tests/model calls run. Per-model/tool checkpoint integration remains unfinished.

### Chat research submission review — 2026-10-30

Chat now builds ResearchInput after message storage and uses start_with with the saved message ID as workflow ID.
Stream captures use the owned service clone and propagate submission failure.
Remaining wiring errors by inspection: obsolete researcher_service constructor dependency, missing research argument in create_app, and missing ServiceError import in app/dbos.rs.
No compilation or runtime checks run; submission is not marked verified.
Research model/tool calls still lack DBOS steps.

### Research submission wiring compiles — 2026-10-30

Confirmed obsolete direct researcher dependency is removed from ConversationService.
App passes the registered research handle and DBOS setup imports ServiceError.
Application cargo check passes with warnings after dependency compilation.
No workflow or paid model calls run. Runtime research submission remains unverified.
Next: checkpoint individual model/tool operations; process_message still uses plain agent.prompt.
Outdated DBOS startup test path/signature and full-suite verification remain pending.

### Researcher step-loop starting point — 2026-10-30

Inspected installed Rig AgentRun and AgentRunStep APIs.
Next user task: replace the opaque prompt driver with an explicit three-branch stepping loop.
Done returns the final output; CallModel and CallTools need checkpointed implementations before execution.
No production code changes or model calls run. This is scaffolding guidance, not completed durable research.

### Model-request preparation lesson — 2026-10-30

User adds AgentRun loop; CallModel and CallTools branches remain empty.
Verified installed Agent::tool_definitions(None) and CompletionRequest builder methods.
Next task: obtain existing agent tool definitions and build each request from prompt/history, researcher preamble, and those definitions.
The model call, DBOS checkpoint, ModelTurn feedback, and tool branch remain unfinished.
No code changes, model calls, or compilation run for this guidance.

### Durable researcher error boundary — 2026-10-30

Request construction now includes existing tool definitions, history, and preamble.
Verified CompletionResponse supports serialization for DBOS checkpoints.
Before adding DBOS steps, process_message must preserve DBOS control errors rather than flattening them into ServiceError.
Next guidance changes its return to dbos::Result<String, ServiceError>, removes the outer Application mapping, and maps Rig errors through ServiceError explicitly.
No implementation edits, compilation, or model execution performed.

### Model checkpoint guidance — 2026-10-30

Researcher now returns dbos::Result and maps AgentRun errors through ServiceError.
The app workflow still wraps that DBOS error in Application; remove that redundant mapping.
Next task: wrap the single model call in dbos::step, clone its request per attempt, and preserve DBOS errors through ?.
Response feedback and tool execution remain unfinished. No code edits or model calls performed.

### Model request checkpoint implemented — 2026-10-30

CallModel now invokes the provider inside dbos::step named research_model.
The request is cloned per attempt; serializable model responses are checkpointed and DBOS errors propagate intact.
Removed the redundant Application error wrapper from the registered workflow.
Application cargo check passes with warnings. No workflow or paid model calls run.
Research remains incomplete: the response is not fed into AgentRun, and CallTools is empty.
Do not treat current research execution or crash recovery as verified. Next: ModelTurn feedback, then checkpointed tool execution.

### Model response feedback lesson — 2026-10-30

Next user task: convert the saved CompletionResponse to Rig ModelTurn using the advertised tool names.
Feed it into AgentRun::model_response and explicitly fail unknown/disallowed tool calls through Rig's resolution API.
This advances Rig to CallTools or Done without another provider request.
Guidance only; feedback and tool execution are not yet implemented or runtime-tested.

### DBOS researcher integration completed — 2026-10-30

Finished the Rig stepping driver in run_durable_agent, shared by production research and focused integration tests.
Each model request and each tool result has a DBOS step. Recorded responses rebuild Rig state during replay.
Tools execute sequentially, preserve typed result content, return argument errors to the model, and reject unknown tools.
Trusted world/conversation/message IDs still come from the registered research input and tool construction.
The app logs research completion/failure for the existing smoke harness.
Updated the obsolete echo test to verify actual app registration/startup/shutdown without submitting research.

Verification:
- All-target cargo check passes.
- Four real-Postgres tests pass with a local scripted provider: completed-workflow deduplication, DBOS engine restart replay, invalid arguments, and unknown tools.
- The restart test interrupts the final model request after two tool calls are checkpointed. Recovery reuses the initial model response and both tool results.
- Broader non-AI live suite passes: 30 native checks plus the API wrapper; the API wrapper reports 23 passes and five paid skips.
- Eight text tests pass. No external model calls or paid smoke tests run.

Limits remain explicit: the restart test replaces the DBOS engine in one process, not an OS-level kill.
A Memgraph commit and DBOS checkpoint are separate commits; an interrupted write may repeat.
Chat storage and workflow submission are separate commits; atomic handoff remains unfinished.
Visible job status, concurrent research/deletion coordination, and production recovery acceptance remain outside this change.
The existing synthetic ranking case is not rerun or changed.
See [DBOS integration results](../tests/artifacts/dbos-research-2026-10-30.md).
