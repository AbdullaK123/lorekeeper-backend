# Lorekeeper — Project Plan

Updated: 2026-11-03.

## Extensive description browser verification — 2026-11-03

All 28 Playwright checks pass in 192.75 seconds on a rebuilt local debug API.
A fresh QA world receives 20 paid chat turns and background research, plus two description rewrites.
Accepted age 41 now agrees with the description, which retains unrelated facts and removes age 34.
Both threads answer 41. Fifteen extended questions preserve the full entity and source IDs after research completes.
Repeated acceptance, resolved-conflict reads, exact history after reload, mobile inspection, and displayed errors pass.
No uncaught browser errors occur. The old stale-description finding is fixed in this tested scenario.
Evidence: [extensive browser summary](../tests/artifacts/playwright-extensive-description-2026-11-03/summary.md).
No full suite rerun or Docker rebuild occurs. The temporary API stops; QA evidence remains.
Next: verify container packaging. Concurrent edits, search indexing, and broader conflict targets remain unchecked.
All story requirements and epic acceptance states stay unchanged.


## Stale embedding fix — 2026-11-03

A free Memgraph test reproduces the bug: corrected descriptions retain the old stored search vector.
WorldService now embeds the corrected description before opening the acceptance transaction.
Entity::embedding_text shares the name/kinds/description format with researcher graph writes.
The repository saves the replacement vector, description, property, sources, and decision together.
Embedding failure returns before graph writes. Repeated acceptance keeps the first committed vector.
Verification: all four conflict regressions pass in 1.34 seconds. All-target offline compilation, formatting, and whitespace checks pass.
The rebuilt image includes this fix. All 16 dev/prod runtime checks pass, including clean exit 0.
Image: lorekeeper-backend:verified-20261103. No Compose rollout or hosted deployment occurs.
Evidence: [image and embedding verification](../tests/artifacts/docker-verification-2026-11-03/summary.md).
No paid browser rerun or full suite rerun occurs for this change.
Concurrent-edit protection and backfilling earlier accepted entities remain outside this change.
Frontend work stays deferred. Story scope and epic acceptance stay unchanged.

## Completion rule

The project is done when all 15 [epics](user_stories.md) pass their acceptance checks.
All 69 stories remain required. Delivery stages set the order, not a smaller product scope.

The [product vision](product_soul.md) defines the audience: writers who need reliable memory for large fictional worlds.
Canon means facts the writer establishes or confirms.

Backend routes alone do not complete a user story.
The client must expose the required behavior, including errors, source links, and graph views.
This repository contains backend work. The plan also names required client work.

No epic currently has recorded acceptance for its complete user workflow.
Working foundations remain valuable and should stay in place during changes.

## Planning record — 2026-11-03

Project instructions now require PM updates before every final response.
This turn changes documentation only. Product scope and delivery priorities stay unchanged.
The latest browser evidence supports one canon scenario, not complete epic acceptance.

## Current foundations

| Area | Implemented | Remaining limits |
| --- | --- | --- |
| Accounts | Signup, login, logout, sessions, current user | Duplicate signup now returns 409; complete privacy acceptance remains |
| Worlds | Create, list, read, update, delete; owner checks | Request text validation is enforced; deletion can race with background lore writes |
| Conversations | Create, list, resume, delete; streamed replies | Long histories, interrupted streams, and concurrent turns need defined behavior |
| Message storage | User and assistant messages save together before background research starts | Research status and retry APIs are built; chat/job handoff and fork/mapping persistence remain separate commits |
| Lore storage | Entities, relationships, conflicts, search, graph repository methods | Message sources persist on entities, relationships, and conflicts; broader identity acceptance and write coordination remain |
| Retrieval | Text and vector search, neighbor properties, relationship conditions | Conflict lookup passes five fresh-conversation checks; broader coverage remains |
| Graph APIs | Scoped graph, entity, neighbor, relationship, and conflict reads; supported conflict decisions | Product graph views and broader conflict targets remain |
| Sources and recovery | Source-message reads, research status, retry submission and current-attempt tracking | Client flows, crash commit gaps, and mapping deletion cleanup remain |
| Container packaging | Multi-stage playground/API build, non-root runtime, dev-only static files, tested graceful stop | Hosting, migrations during release, backups, and deployed acceptance remain |
| Logging | Structured request/service/repository/workflow logs and privacy capture tests | Production monitoring and alerting remain |
| Cleanup | Transactional deletion jobs, scheduled graph cleanup, retry after failure | Shutdown and competing background writes need further checks |
| Researcher | [Prompt](../prompts/researcher.md), scoped search/read/write/conflict tools, background runtime wiring | DBOS model/tool checkpoints implemented; interrupted writes and concurrent recovery still need acceptance |

The runtime uses a background tool-using researcher after successful chat storage.
Chat has read-only search access. The old structured extraction pipeline is removed.
Researcher jobs now run as DBOS workflows with model and tool checkpoints in Postgres.
Local scripted-provider tests verify DBOS engine restart replay. Atomic handoff and interrupted graph writes remain unfinished.

## Test baseline

Latest recorded verification on 2026-11-03:

- All-target offline compilation passes with warnings.
- 44 free native regressions pass in 12.94 seconds.
- Nine research retry/status HTTP tests pass in 9.475 seconds.
- The broader unit run has 11 passes and one duplicate-row retrieval-ranking failure.
- The accepted-canon browser regression passes all 10 steps in a fresh QA world.
- Both existing and fresh conversations answer the accepted age; research preserves saved entity data.
- Browser fix attempts use 13 paid chat turns and one embedding diagnostic.
- The full unit suite is not rerun. The Docker image still needs rebuilding.

These are separate scoped runs, not a combined full-suite pass count.
The duplicate-row assertion remains failing; no current query occurrence is demonstrated and no fix is scheduled.
Earlier paid smoke results remain in [test findings](../tests/REPORT.md) and their artifacts.
Source preservation, conflict targets, graph APIs, and restart replay have targeted evidence.
This evidence does not prove every identity case or exactly-once writes across databases.

## Epic completion checks

### Epic 1: Keep My Worlds Private and Separate

**Stories:** US-001, US-002, US-003, US-004.

**Built:** Account sessions, world management, owner checks, and queued graph deletion.
World deletion also removes its conversations in Postgres.

**Remaining:**

- Complete account and world screens, including useful validation errors.
- Verify isolation across every API and researcher tool.
- Prevent background research from recreating a deleted world's lore.
- Define how the client shows pending deletion and cleanup failures.

**Done when:** Two writers cannot access each other's data through any supported path.
Deleting a world removes its conversations and lore, even during active research or a restart.

### Epic 2: Build My World Through Conversation

**Stories:** US-005, US-006, US-007, US-008, US-009.

**Built:** Streamed chat, saved history, separate conversations, and read-only lore search during replies.
The prompts guide natural conversation and follow-up questions.

**Remaining:**

- Complete conversation navigation, note pasting, streaming, and resume behavior in the client.
- Define supported note size and history limits.
- Handle long conversations without losing relevant earlier context.
- Define concurrent turn ordering, interrupted replies, and safe retry behavior.
- Test follow-up questions against real worldbuilding cases.

**Done when:** A writer can paste notes, discuss them, leave, and resume without lost or duplicated turns.
Follow-up questions use the conversation and saved lore without inventing canon.

### Epic 3: Remember My Lore Without Manual Filing

**Stories:** US-010, US-011, US-012, US-013, US-014, US-015.

**Built:** Tool-using researcher, exact-ID batch writes, full reads, conflict reporting, message sources,
DBOS checkpoints, status polling, and retry APIs. Retry state tracks the current workflow attempt.

**Remaining:**

- Verify contextual identity through the tool-using researcher with behavioral evaluations.
- Preserve old facts, minor details, relationship conditions, and sources during updates.
- Resolve identity through context; ask for clarification when evidence remains unclear.
- Support distinct subjects with the same name without silently merging them.
- Connect the built status/retry APIs to client feedback.
- Close chat/job and fork/mapping commit gaps; define cleanup for mapping rows after message deletion.
- Protect updates against concurrent jobs and repeated processing.

**Done when:** Restarting or retrying research loses no accepted lore and creates no duplicate facts or relationships.
Ambiguous names never cause silent writes to the wrong subject.
The writer can see and retry failed saves.

### Epic 4: Find Answers I Can Trust

**Stories:** US-016, US-017, US-018, US-019, US-020.

**Built:** Natural-language retrieval, combined ranking, entity/neighbor properties, relationship conditions,
message source IDs, and an authorized source-message API.

**Remaining:**

- Complete claim-level answer citations and their client navigation.
- Link answer claims to readable source statements in the client.
- State unknown details clearly and distinguish unresolved claims from established facts.
- Test retrieval with indirect wording, minor details, and misleading nearby matches.

**Done when:** A writer can verify factual answers against their earlier statements.
Answers preserve conditions and direction, and identify missing evidence.

### Epic 5: Keep Control of Canon and Contradictions

**Stories:** US-021, US-022, US-023, US-024, US-025, US-026, US-027.

**Built:** Source-aware conflict records, scoped list/detail APIs, exact property-key conflict reporting,
and keep-existing/accept-new decisions. Accept-new updates the entity property and resolves the conflict atomically.

**Remaining:**

- Connect conflict read/decision APIs to client review.
- Extend supported conflict targets beyond existing entity properties.
- Define unclear-reference review and consistent changes across affected relationships.
- Keep hypothetical statements and unapproved assistant suggestions outside canon.
- Route unresolved identity questions back to the writer before attaching facts.

**Done when:** A writer sees the evidence, decides what remains canon, and verifies the resulting lore.
Unresolved issues remain visible across sessions without silently replacing established facts.

### Epic 6: See How My World Connects

**Stories:** US-028, US-029, US-030, US-031.

**Built:** Owner-scoped graph, entity, search, neighbor, relationship, and conflict HTTP reads.
Filtered pagination and populated graph fixtures have targeted tests.

**Remaining:**

- Build the visual graph and entity inspection views in the client.
- Support following connections across entity types.
- Show relationship direction, conditions, details, and source links.
- Keep large worlds usable through bounded loading and navigation.

**Done when:** A writer can explore their world visually and inspect a character's connections with supporting details.

### Epic 7: Reason About Consequences Within My World

**Stories:** US-032, US-033, US-034, US-035.

**Built:** Search and graph context provide a starting point.
Dedicated consequence reasoning has no recorded acceptance yet.

**Remaining:**

- Keep hypothetical reasoning separate from automatic canon writes.
- Retrieve the relevant chain of relationships and its conditions.
- Explain inferred consequences with the supporting facts and source links.
- Identify missing premises rather than inventing world rules.
- Require explicit writer confirmation before saving a proposed change.

**Done when:** A hypothetical trade disruption produces a traceable explanation grounded in saved dependencies.
The answer labels inferences and unknowns, and leaves canon unchanged without confirmation.

### Epic 8: Find Gaps and Choose What to Develop

**Stories:** US-036, US-037, US-038, US-039.

**Built:** Saved lore and search provide inputs. Dedicated gap review remains unbuilt.

**Remaining:**

- Compare related entities using their saved facts.
- Identify uneven detail without treating every difference as a mistake.
- Gather open questions and let the writer return to them later.
- Show the evidence behind each suggested gap.
- Present gaps as questions without generating canon answers.

**Done when:** A writer can compare factions, review missing details, and return to open questions later.
The system preserves the writer's choice about what needs development.

### Epic 9: Access and Recover My Account

**Stories:** US-040, US-041, US-042, US-043, US-044.

**Status:** Required scope; complete implementation and acceptance are not recorded.
Password authentication and sessions provide an existing foundation.

**Remaining:**

- Extend the existing account model with verified identities and recovery records.
- Build verification, OAuth, linking, and recovery flows with client feedback.

**Decisions before build:** Provider selection, account linking rules, token expiry, and recovery for accounts without passwords.

**Done when:** A writer can sign in, recover access, and link identities without exposing or duplicating their worlds.

### Epic 10: Manage My Subscription and Payments

**Stories:** US-045, US-046, US-047, US-048, US-049, US-050.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Add customer, subscription, billing-event, and access records.
- Build checkout, billing management, event processing, reconciliation, and plan screens.

**Decisions before build:** Prices, currencies, taxes, refunds, trial policy, plan changes, and payment-failure access rules.

**Done when:** Payment state reliably controls access, and writers can manage billing without losing their lore.

### Epic 11: Understand and Control Usage

**Stories:** US-051, US-052, US-053, US-054.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Build usage records, reservations, access checks, and worker budget enforcement.
- Add usage screens, renewal information, and clear limit responses.

**Decisions before build:** Allowance units, reset periods, retry charging, queued-work policy, and hard spending ceilings.

**Done when:** Concurrent chat and background jobs respect limits, and writers see accurate usage and renewal information.

### Epic 12: Receive Essential Service Emails

**Stories:** US-055, US-056, US-057.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Build templates, queued delivery, provider integration, and delivery status handling.
- Connect account and billing events to the required messages.

**Decisions before build:** Email provider, sender domain, required notices, and which system owns each billing notice.

**Done when:** Essential messages arrive reliably, and failures remain visible and recoverable.

### Epic 13: Operate and Recover the Service

**Stories:** US-058, US-059, US-060, US-061.

**Status:** Required scope; complete implementation and acceptance are not recorded.
Containers, health response, cleanup retries, and partial shutdown handling provide existing foundations.

**Remaining:**

- Add health checks, structured failure reporting, alerts, and release automation.
- Write incident, restore, migration, and rollback procedures.

**Decisions before build:** Hosting, alert ownership, acceptable downtime, acceptable data loss, and backup retention.

**Done when:** Operators detect failures and prove recovery within agreed downtime and data-loss targets.

### Epic 14: Protect and Manage My Data

**Stories:** US-062, US-063, US-064, US-065, US-066.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Build export and account-deletion jobs with visible progress and safe retries.
- Add access restrictions, rate limits, administrative audit records, and accurate policy pages.

**Decisions before build:** Export format, retention periods, deletion confirmation, provider data handling, and launch policy requirements.

**Done when:** Writers can export and delete data, and published privacy behavior matches deployed handling.

### Epic 15: Get Help and Resolve Service Problems

**Stories:** US-067, US-068, US-069.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Add support links, useful error references, and operational lookup tools.
- Add controlled retry actions and administrative audit records.

**Decisions before build:** Support channel, response expectations, administrative roles, and rules for authorizing private-content access.

**Done when:** Writers can report failures, and authorized operators can investigate and recover work without broad data access.

## Delivery order

### Stage 1: Define the research boundary

**Status:** Runtime boundary and tools are built; broader identity acceptance remains.

**Supports:** Epics 3, 4, and 5.

The researcher prompt and initial runtime tools are implemented.
The old extraction pipeline is removed. Message references and scoped read/write contracts are implemented.
Broader identity behavior and explicit unresolved-reference workflows still need acceptance.

1. Define stable entity references and source message references.
2. Define search and full-read results, including facts, relationships, conditions, conflicts, and sources.
3. Decide how distinct same-name subjects fit the current unique-name storage rule.
4. Define create, update, link, unlink, conflict, and duplicate-merge tool contracts.
5. Define structured results for confirmed saves, unresolved references, and failures.

Tools enforce world ownership and exact targets.
The researcher interprets meaning and ambiguity within those boundaries.
Do not give the conversational agent unrestricted lore writes.

**Exit check:** Example tool calls represent every researcher prompt example without guessing a target or losing source evidence.

### Stage 2: Build one researcher workflow

**Status:** Implemented and smoke-tested. The items below describe required acceptance coverage, not missing tool scaffolds.

**Supports:** Epics 3 and 5.

1. Add search and full-read tools first.
2. Add entity writes that preserve earlier facts and sources.
3. Add exact-target relationship writes and explicit retractions.
4. Add conflict reporting and safe duplicate repair.
5. Run the researcher against isolated test worlds.

Start with one saved statement, one entity, and a verified read-back.
Then add updates, alternate names, relationships, contradictions, and corrections as separate slices.

**Exit check:** Behavioral tests cover shared aliases, distinct same-name subjects, preserved facts, and repeated processing.
They also cover qualifiers, explicit corrections, speculation, and unresolved references.

### Stage 3: Make automatic research reliable

**Supports:** Epics 1, 2, and 3.

1. Save the research job with the conversation changes in one Postgres transaction.
2. Run jobs with bounded work, durable status, retries, and restart recovery.
3. Prevent competing jobs from overwriting newer accepted facts.
4. Coordinate deletion with research so stale work cannot restore deleted lore.
5. Connect the built research status/retry APIs to the client and close their recorded commit gaps.
6. Complete behavioral acceptance of the researcher now wired into the runtime.
7. Keep obsolete extraction and alias logic removed; add regression cases against the active tools.

A one-time world existence check does not protect a later graph write from deletion.
The design must cover that interval and retries after partial writes.

**Exit check:** Failure and restart tests prove recovery, visible failure, and complete deletion during active research.

### Stage 4: Complete trustworthy answers and canon review

**Supports:** Epics 2, 4, and 5.

1. Fix retrieval omissions and verify ranking behavior.
2. Carry sources from research through retrieval into answer links.
3. Add conflict listing, inspection, resolution, and unclear-reference review.
4. Complete client flows for source inspection, corrections, and unresolved issues.
5. Verify long conversations, note limits, and follow-up behavior.

**Exit check:** A writer can save lore, question it, inspect its sources, and resolve a contradiction.
The resulting answers reflect the writer's decision.

### Stage 5: Complete graph exploration

**Supports:** Epic 6.

1. Use the built scoped graph, entity, neighbor, relationship, and conflict APIs.
2. Build the visual graph and connection inspection views.
3. Add source navigation and bounded loading for large worlds.

**Exit check:** Complete every Epic 6 workflow using the client against a representative world.

### Stage 6: Complete reasoning and gap review

**Supports:** Epics 7 and 8.

1. Add hypothetical reasoning with evidence paths and missing-premise reporting.
2. Verify that hypothetical conversations do not enter canon automatically.
3. Add grounded comparisons, gap questions, and a persistent open-question view.
4. Test these behaviors with incomplete and conflicting lore.

**Exit check:** Consequences remain traceable, unknowns remain explicit, and all canon changes require writer intent.

### Stage 7: Complete SaaS launch workflows

**Supports:** Epics 9–15.

1. Decide sign-in providers, plans, allowances, email delivery, and hosting.
2. Build account verification and recovery with reliable service emails.
3. Build usage accounting before enabling paid model workloads.
4. Add Stripe checkout, billing events, reconciliation, and account access rules.
5. Complete export, account deletion, support, and restricted operational tools.
6. Prove monitoring, backup recovery, release rollback, and published policy accuracy.

Build essential operational controls alongside earlier stages, before dependent features accept real users.
Do not treat this stage as permission to delay recovery, privacy, or spending controls.
Use the [lifecycle design](design.md) for each epic’s release and maintenance checks.

**Exit check:** Account, payment, limits, email, support, and recovery workflows pass deployed acceptance.

### Stage 8: Close every epic

**Supports:** All epics.

1. Preserve verified duplicate-signup handling and typed request validation.
2. Complete account, world, conversation, and recovery screens.
3. Test ownership across every new route and tool.
4. Test interrupted streams, concurrent turns, worker recovery, and deletion races.
5. Test deployment shutdown signals, including SIGTERM, and active job behavior.
6. Verify fresh migrations, backup recovery, and useful failure reporting.
7. Record acceptance evidence for each of the 69 stories.

Work on foundation defects alongside earlier stages when they block a workflow.
Do not postpone privacy or data-loss defects until final acceptance.

**Exit check:** Every story passes its writer-facing workflow, including failure and recovery where relevant.
No required epic moves into a later version to declare the project complete.

## SaaS launch decisions

The added scope is required. Provider choices and commercial rules remain open.
No price, allowance, OAuth provider, or hosting vendor is selected by this document.
Stripe is the intended billing integration.

Choose these rules before building their dependent flows:

- Supported OAuth providers and account-linking behavior.
- Plan prices, included usage, renewal periods, cancellation, and failed-payment access.
- Email provider, sender identity, and ownership of each notice.
- Hosting, recovery targets, retention, privacy disclosures, and support ownership.

Record each choice in the matching design epic and verify current provider requirements during implementation.
The earlier code-size estimate excludes this added scope and needs revision.

## Immediate next pieces

Next-step review: verify the current changes in a rebuilt Docker image.
Check health, playground access under ENV=dev, asset blocking under ENV=prod, and clean shutdown.
The 28-step browser run verifies the local debug API, not the release image.
No implementation changes or tests run in this planning turn. Scope and epic acceptance stay unchanged.
User direction: focus on backend work. Defer the actual frontend, including graph visualization.
Keep all frontend stories in scope for later; do not schedule them as the next implementation slice.
Release image verification now passes all 16 checks after the description and embedding fixes.
Stale embeddings are reproduced and fixed for future acceptance writes. Existing stale vectors still need a repair plan.
Concurrent-edit protection and research/deletion coordination remain backend gaps.
This planning turn changes no application code and runs no tests. Epic acceptance stays unchanged.


Next verification: rerun the fresh-world Playwright acceptance flow with an explicit description assertion.
Require age 41 in saved properties, removal of the old age claim from the description, and agreement in both conversation answers.
After question research completes, require the accepted entity and source IDs to remain unchanged.
The existing browser harness does not fail on stale descriptions; strengthen that check before running.
This planning turn runs no tests or paid calls and changes no application code.
Scope and epic acceptance stay unchanged. Concurrent-edit protection remains separate unfinished work.


Implemented: accept_conflict now receives the generated description and saves it in the existing graph transaction.
Properties, description, message sources, and the conflict decision share that transaction.
The service passes model output text and maps model-client environment errors into ServiceError.
The free repository regression now checks description persistence and unchanged description on repeated acceptance.
Verification: focused free database regression passes (1 test, 1.38 seconds). All-target offline compilation passes with warnings. Formatting and whitespace checks pass.
It no longer exercises WorldService, which now makes a paid model call.
Stale-input protection and description-based search indexing remain unchecked.
No browser rerun or paid calls occur in this implementation turn. Scope and epic acceptance stay unchanged.


Transaction guidance: reuse GraphRepository::accept_conflict and its existing commit/rollback handling.
Add the rewritten description as an argument and include e.description in the existing entity SET query.
The property, description, sources, and conflict decision then commit in the same graph transaction.
The service now generates new_description, but the repository does not save it yet.
A stale-input check remains needed before claiming concurrent-edit protection.
No application code changes or tests run during this guidance turn. Scope and epic acceptance stay unchanged.


Request guidance: use a tool-free Rig AgentBuilder with the configured chat model.
Send the old description and accepted field/value as JSON data; request only revised description text.
Await prompt to obtain a String. Existing ServiceError conversions cover provider and prompt errors.
Prefer injecting the configured model from bootstrap instead of creating clients inside WorldService.
The current acceptance method contains an unfinished request expression. No application code changes or tests run here.
The atomic description write and stale-snapshot check remain unimplemented. Scope and acceptance stay unchanged.


Proposed description fix: WorldService reads the entity and conflict, then requests one focused description rewrite.
The model receives the existing description and accepted correction, with instructions to preserve unrelated facts.
Run that model call before opening the write transaction.
Extend accept_conflict to save the corrected property, description, sources, and decision together.
Reject a stale entity snapshot instead of overwriting changes made during the model call.
If rewriting fails, leave the conflict unresolved and the entity unchanged.
This is a proposal, not an implemented or verified change. No code or tests run this turn.
Scope and epic acceptance stay unchanged. Description-based search indexing also needs checking during implementation.

Next task: choose how accepting a conflict also updates the entity description.
The observed case saves age 41 while the description still says 34 years old.
Acceptance check: property, description, and answers agree after acceptance; question research preserves that result.
The implementation approach remains undecided. No code changes or tests run in this planning turn.
Product scope and epic acceptance stay unchanged.

Fresh-world recheck: all 10 browser steps pass again; both threads answer the accepted age 41.
Research preserves the accepted entity. The description still says 34 years old.
See [recheck evidence](../tests/artifacts/playwright-recheck-2026-11-03/summary.md).
No application changes or full suite rerun occur. Story scope and epic acceptance stay unchanged.

Next-step review: rebuild the API image, then verify health and environment-based playground serving.
The latest canon checks use the local debug API; they do not verify the current container image.
Then address the observed stale description after conflict acceptance.
This planning turn runs no application tests and changes no delivery acceptance states.

The 2026-11-03 canon rollback now has a passing [browser regression](../tests/artifacts/playwright-canon-decisions-2026-11-03/summary.md).
Current properties and resolved decisions override stale history; questions preserve saved lore in that scenario.
Description text can still lag behind an accepted property. Define summary consistency before declaring full canon acceptance.

The researcher pipeline, graph read APIs, supported conflict decisions, source lookup, logging,
and research status/retry endpoints are implemented. Do not restart the extraction redesign.

1. Connect product client flows for graph browsing, source inspection, conflict decisions, and research recovery.
2. Choose the next backend slice: long-history limits, broader conflict targets, or deletion/research coordination.
3. Close recorded recovery commit gaps and mapping-row cleanup before claiming full recovery acceptance.
4. Keep SaaS account, billing, usage, email, privacy, support, and operations scope visible.

The next implementation task needs a bounded acceptance check. No new task is started by this documentation update.
