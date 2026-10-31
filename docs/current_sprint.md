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

### Ownership and clone cleanup — 2026-10-30

Audited production clone sites, focusing on research, chat, auth, retrieval, and graph-result formatting.
Removed per-stream-event user-message clones. Chat moves saved message/history values into final processing and research input.
Reused the world ID string instead of formatting it twice. Final response terminates the stream after submitting research.
Research agent construction moves IDs to their final owners. Model requests are built only inside executing DBOS steps.
Tool steps borrow the pending call instead of cloning the entire call; owned checkpoint identifiers/content still copy as required.
DBOS registration shares ResearcherService through Arc instead of copying its prompt/configuration for every workflow.
Login borrows the stored password hash, returns its owned User, and copies the UUID directly.
Retrieval consumes search result lists instead of cloning every entity through concat; ranking rules are unchanged.
Graph update response views borrow saved IDs/names instead of copying strings before serialization.
Retained clones needed for owned Rig requests, checkpoint results, independent tool contexts, and shared async handles.

Verification: all-target cargo check passes. The live suite passes 30 native checks plus the API wrapper (23 API passes, five paid skips).
All four DBOS replay tests pass within that run. Pure tests report 11 passes and the unchanged synthetic duplicate-ranking failure.
No external model calls run. Paid chat streaming is not re-smoked for this cleanup.

### Manual research loop walkthrough — 2026-10-31

Read the current implementation before explaining AgentRun's role.
Walkthrough starts with its stored conversation state, next_step requests, and model-call budget.
Distinguishes Rig's in-memory state tracking from DBOS model/tool result persistence.
No code changes or tests run. Detailed branch walkthrough continues as a learning task.

### Remaining product scope audit — 2026-10-31

Compared user stories, plan, controller routes, and graph repository methods.
Current public routes cover accounts/sessions, worlds, and conversations. Graph browsing and conflict-review routes are absent.
Graph visualization is the next proposed product slice: world-scoped graph data, entity search/detail, bounded neighbor expansion, and client views.
Further product work includes source navigation, conflict resolution, research status, reasoning/gap review, and SaaS epics 9–15.
Repository methods are foundations rather than completed authorized HTTP workflows.
Some older epic checklists still list completed source storage/context formatting as remaining; use current verified evidence when planning.
No implementation changes or tests run for this audit.

### WorldService method inventory — 2026-10-31

Reviewed WorldService and graph repository reads before defining the next API slice.
Proposed read methods: get_graph, list_entities, search_entities, get_entity, get_entity_neighbors, get_relationship, list_conflicts, and get_conflict.
Proposed follow-up mutation: resolve_conflict with an explicit writer decision; toggling resolved alone does not apply that decision to canon.
All methods require user/world scope and ownership verification before graph access.
GraphRepository is the new service dependency. Bounded graph/list queries and individual relationship lookup need repository work.
Source-message navigation stays with conversation APIs; billing/account operations stay outside WorldService.
Inventory only; no methods or stubs implemented and no tests run.

### Pagination wrapper review — 2026-10-31

User adds shared PageResponse<T> with data: T and next_cursor: Option<String>, exported through shared.
The wrapper is valid: T describes the whole page payload, allowing a vector for lists or nodes/edges for graph data.
Use PageResponse<Vec<EntityResponse>> for entity pages; keep pagination metadata in the wrapper.
No changes or runtime tests performed. World graph query/response models remain unimplemented.

### Graph API model scaffolding — 2026-10-31

Added GraphQuery, EntityListQuery, EntitySearchQuery, NeighborQuery, and ConflictListQuery in data/models/graph.rs.
Queries declare a default limit of 50, valid limits 1–100, bounded cursor/filter strings, and validated SearchQuery text.
Added GraphResponse, EntityResponse, RelationshipResponse, and ConflictResponse with source references and directed edge IDs.
PageResponse<T> remains the single pagination wrapper; GraphResponse contains only nodes and edges.
All-target cargo check passes with warnings. No runtime tests or model calls run for these type definitions.
Remaining: conversions, service/repository methods, cursor implementation, and HTTP query validation wiring.
Conflict-resolution requests remain deferred until the decision contract is defined.

### WorldService read-method scaffolding — 2026-10-31

Added eight async stubs: get_graph, list_entities, search_entities, get_entity, get_entity_neighbors, get_relationship, list_conflicts, and get_conflict.
Signatures use user/world UUIDs, record UUIDs, borrowed query models, and typed paginated responses.
WorldService now receives GraphRepository; app constructor wiring is updated.
Bodies contain explicit todo markers and are not connected to HTTP routes.
All-target cargo check passes with warnings. No runtime tests or stub/model execution run.
Ownership checks, conversions, queries, pagination, and HTTP wiring remain implementation work.
Conflict resolution stays deferred until its request contract is defined.

### Graph response conversions — 2026-10-31

Added owned From conversions for Entity, Relationship, and Conflict into their API response types.
GraphResponse accepts separate entity/relationship lists or existing neighbor query tuples.
Paginated record lists convert with an explicit next_cursor; the shared PageResponse accepts any payload/cursor pair.
Conversions move fields and collections without cloning; graph order and input rows are preserved.
All-target cargo check passes with warnings. No runtime tests or model calls run for these mechanical conversions.
WorldService read stubs, pagination queries, and HTTP endpoint wiring remain unfinished.

### Graph repository pagination — 2026-10-31

Added GraphRepository::get_graph with stable entity-ID ordering and a last-ID cursor.
Each page returns 1–100 entities and relationships whose endpoints both belong to that page and world.
An extra entity probe determines whether another page exists. Empty and final pages return no cursor.
Added graph-page response conversion and invalid-input error mapping to HTTP 422.
Added four real Memgraph tests for page boundaries, direction, world scope, and invalid input.
Tests do not execute: compilation stops at the unfinished WorldService::get_graph return path (E0317).
Repository-wide formatting check also fails; no broad formatting changes applied.
Remaining: finish the service call, rerun graph_page_ tests, then add HTTP routes and query validation.

### World graph service review — 2026-10-31

Reviewed the user's completed WorldService::get_graph implementation.
World ownership is checked before graph access; repository errors propagate and results convert to the response.
All four graph_page_ tests pass against real Memgraph fixtures (1.61 seconds).
Suggested borrowing the cursor with as_deref() and using the existing page conversion; service code stays unchanged.
HTTP routes and query validation wiring remain unfinished. Service ownership behavior has no new dedicated runtime test.

### Entity listing approach — 2026-10-31

Reviewed list_entities and EntityListQuery. Existing get_entities_by_world has no pagination or kind filter.
Next repository method needs world scope, optional kind filtering, stable ID ordering, and limit-plus-one cursor detection.
Reuse the graph-page pagination pattern without loading relationships. No implementation changes or tests for this planning step.

### Entity listing repository query — 2026-10-31

Added GraphRepository::list_entities(world_id, limit, cursor, kind).
Uses world-scoped, parameterized Cypher with optional exact kind filtering and stable ID ordering.
Validates limits and cursor syntax; fetches one extra entity to detect the next page.
Returns entities and an optional next cursor for the existing response conversion.
git diff --check passes for the repository file. Confirmed neo4rs supports optional query parameters.
No runtime tests run for this method. The service list_entities body still lacks its final return value.
Remaining: finish the service call, verify filtered pagination against Memgraph, and wire HTTP routes.

### Entity listing review findings — 2026-10-31

The current service call uses an undefined kind variable; it must borrow query.kind.
The page conversion applies to the full repository tuple, not Vec<Entity> alone.
Found an error in the newly added repository filter: Entity.kind is a list, so filtering needs membership rather than scalar equality.
These issues remain unfixed and unverified. Resolve them before moving to search_entities.

### Entity kind filter correction — 2026-10-31

Changed list_entities to use $kind IN e.kind because entity kinds are stored as lists.
The null filter still allows all kinds. Query values remain parameterized.
git diff --check passes for the repository file. No runtime tests run for this correction.
Filtered pagination still needs Memgraph verification; service fixes listed above are not rechecked here.

### Entity listing service follow-up — 2026-10-31

Confirmed list_entities now borrows query.kind and query.cursor and converts the full repository tuple with result.into().
The previously reported undefined variable and vector-conversion issues are corrected by inspection.
No compile or runtime checks run for this follow-up. Filtered pagination verification remains pending.

### Reusable text-search limit — 2026-10-31

Added a u32 limit argument to fts_search_entities and bound it as a Cypher parameter.
The existing agent retrieval caller passes 10. Removed cursor from EntitySearchQuery for top-match search without pagination.
WorldService::search_entities remains a stub for the next implementation step.
All-target offline cargo check passes with existing warnings. Changed-file whitespace check passes.
No runtime tests or external model calls run for this mechanical change.

### Reusable vector-search limit — 2026-10-31

Added a u32 limit argument to vector_search_entities; agent retrieval passes 10.
Cypher binds the output limit and candidate count. Candidate count stays at least 50 and grows for larger limits.
All-target offline cargo check and changed-file whitespace check pass, with existing compiler warnings.
No runtime search tests or external model calls run. World-scoped filtering still occurs after global vector candidate selection.

### Paginated neighbor repository query — 2026-10-31

Added get_entity_neighbors_page with limit and cursor arguments; existing agent neighbor reads stay unchanged.
Pages by relationship ID so multiple edges between the same entities remain separate records.
Filters neighbor and relationship world scope, optional relationship type, and kind-list membership.
Fetches one extra row to detect the next cursor and validates limit/cursor inputs.
Returns (neighbor/relationship pairs, next_cursor). The limit counts edges rather than unique neighbor nodes.
Changed-file whitespace check passes. No compilation or runtime checks run for this addition.
Remaining: wire the world service to the paginated method, convert its response, and verify pagination against Memgraph.

### Neighbor service conversion review — 2026-10-31

Confirmed the service now passes pagination and filters to get_entity_neighbors_page and awaits its result.
Missing conversion: (Vec<(Entity, Relationship)>, Option<String>) into PageResponse<GraphResponse>.
The conversion can reuse GraphResponse::from(neighbor pairs) and preserve next_cursor.
No code changes or compile/runtime checks for this review. Source entity inclusion and duplicate neighbor nodes remain response-design considerations.

### Paginated neighbor response conversion — 2026-10-31

Added From<(Vec<(Entity, Relationship)>, Option<String>)> for PageResponse<GraphResponse>.
Reuses the existing neighbor-pair conversion and moves the cursor unchanged, without cloning.
All-target offline cargo check passes with warnings. Changed-file whitespace check passes.
No runtime tests run. Neighbor query pagination still needs Memgraph verification.

### Remaining world-service read methods — 2026-10-31

Completed get_relationship, list_conflicts, and get_conflict with world ownership checks and typed responses.
Missing relationships and conflicts return NotFound. Added world-scoped relationship lookup and paginated conflict listing queries.
Conflict pages sort by ID, validate pagination inputs, and support resolved, unresolved, or both states.
Existing agent conflict queries retain their behavior. No WorldService todo stubs remain.
All-target offline cargo check passes with existing warnings; changed-file whitespace check passes.
Two new live tests pass against isolated Postgres and Memgraph (1.11 seconds).
Tests cover owner reads, foreign-user rejection, missing records, conflict page boundaries, filters, scope, and invalid pagination.
The first test run exposed fixture cleanup order; corrected it and reran successfully.
Remaining: HTTP controllers/routes, query validation wiring, and live verification of entity/neighbor pagination.
No external model calls run.

### World graph HTTP routes — 2026-10-31

Follow-up: grouped all world-controller imports at the top and merged repeated module imports.
Controller rustfmt and whitespace checks pass. Import-only change; no runtime tests rerun.

### Application bootstrap extraction — 2026-10-31

Moved repository/service creation, graph constraints, DBOS setup, AppState assembly, and scheduler startup into bootstrap(pool, graph, settings).
bootstrap returns AppState, JobScheduler, and DBOS so existing shutdown ownership stays intact.
create_app retains configuration loading, database connections, router construction, and middleware wiring.
All-target offline cargo check passes with existing warnings. Formatted app.rs; changed-file whitespace check passes.
No runtime tests or model calls run for this mechanical extraction. Existing product verification tasks remain unchanged.

Wired eight GET routes under /world/{world_id}: graph, entities, entities/search,
entities/{entity_id}, entities/{entity_id}/neighbors, relationships/{relationship_id},
conflicts, and conflicts/{conflict_id}.
Handlers extract the authenticated user, parse UUID paths, and call the world service.
Added reusable ValidatedQuery extraction; malformed query values and model validation errors return HTTP 422.
All-target offline cargo check passes with existing warnings. Changed-file whitespace check passes.
Two live API tests pass through the real router (3.544 seconds; Rust wrapper also passes).
Tests cover all eight routes on empty worlds, missing records, foreign-user and anonymous rejection,
and invalid limits, filters, cursors, and search text. No external model calls run.
Remaining: populated entity/neighbor pagination tests and frontend integration.
