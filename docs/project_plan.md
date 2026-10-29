# Lorekeeper — Project Plan

Updated: 2026-10-29.

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

## Current foundations

| Area | Implemented | Remaining limits |
| --- | --- | --- |
| Accounts | Signup, login, logout, sessions, current user | Duplicate signup returns an internal error; complete privacy acceptance remains |
| Worlds | Create, list, read, update, delete; owner checks | Blank names pass validation; deletion can race with background lore writes |
| Conversations | Create, list, resume, delete; streamed replies | Long histories, interrupted streams, and concurrent turns need defined behavior |
| Message storage | User and assistant messages save together before background research starts | Background research lacks durable jobs and visible retry status |
| Lore storage | Entities, relationships, conflicts, search, graph repository methods | Alias collisions, source tracking, and safe updates need work |
| Retrieval | Text and vector search, combined ranking, neighbor lookup | Answer context drops neighbor properties and relationship details |
| Cleanup | Transactional deletion jobs, scheduled graph cleanup, retry after failure | Shutdown and competing background writes need further checks |
| Researcher | [Prompt](../prompts/researcher.md), scoped search/read/write/conflict tools, background runtime wiring | Model behavior, durable jobs, and recovery still need acceptance |

The runtime uses a background tool-using researcher after successful chat storage.
Chat has read-only search access. The old structured extraction pipeline is removed.
Researcher jobs still run in memory; durable recovery remains unfinished.

## Test baseline

The latest recorded [test report](../tests/REPORT.md) contains 72 distinct tests: 65 pass and seven fail.
These are recorded results, not a new test run for this plan.

| Failure | Planned action |
| --- | --- |
| Shared alias selects the wrong entity | Test contextual identity through the researcher tools |
| Alias shadows another entity's canonical name | Require exact saved targets for writes; preserve distinct subjects |
| Neighbor properties disappear from answer context | Include relevant properties in retrieval results |
| Relationship descriptions disappear from answer context | Preserve descriptions, conditions, and direction |
| Duplicate search rows inflate ranking | Confirm production exposure; prevent duplicate evidence from changing rank |
| Duplicate signup returns 500 | Return a clear account conflict response |
| Blank world names pass validation | Confirm the naming rule, then enforce it consistently |

Replace implementation-specific alias tests with behavioral tests when the old pipeline leaves production.
Keep tests proving that facts attach to the correct subject.
Do not remove a failed behavior to make the suite pass.

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

**Built:** Background researcher after saved chat, exact-ID batch writes, full reads, and conflict reporting tools.

**Remaining:**

- Verify contextual identity through the tool-using researcher with behavioral evaluations.
- Preserve old facts, minor details, relationship conditions, and sources during updates.
- Resolve identity through context; ask for clarification when evidence remains unclear.
- Support distinct subjects with the same name without silently merging them.
- Persist research jobs and expose pending, complete, failed, and retry states.
- Protect updates against concurrent jobs and repeated processing.

**Done when:** Restarting or retrying research loses no accepted lore and creates no duplicate facts or relationships.
Ambiguous names never cause silent writes to the wrong subject.
The writer can see and retry failed saves.

### Epic 4: Find Answers I Can Trust

**Stories:** US-016, US-017, US-018, US-019, US-020.

**Built:** Natural-language search, combined search ranking, and graph context for replies.

**Remaining:**

- Store stable references to source messages and carry them through lore updates.
- Return relevant entity properties, neighbor properties, and relationship conditions.
- Link answer claims to readable source statements in the client.
- State unknown details clearly and distinguish unresolved claims from established facts.
- Test retrieval with indirect wording, minor details, and misleading nearby matches.

**Done when:** A writer can verify factual answers against their earlier statements.
Answers preserve conditions and direction, and identify missing evidence.

### Epic 5: Keep Control of Canon and Contradictions

**Stories:** US-021, US-022, US-023, US-024, US-025, US-026, US-027.

**Built:** Conflict records, contradiction checks, and prompt rules for explicit corrections.
Some tests cover preserved canon and contradictions across connected facts.

**Remaining:**

- Preserve both disputed claims and their sources.
- Expose unresolved conflicts and unclear references through APIs and client views.
- Let the writer resolve a conflict or make an explicit correction.
- Apply the decision consistently to saved facts and affected relationships.
- Keep hypothetical statements and unapproved assistant suggestions outside canon.
- Route unresolved identity questions back to the writer before attaching facts.

**Done when:** A writer sees the evidence, decides what remains canon, and verifies the resulting lore.
Unresolved issues remain visible across sessions without silently replacing established facts.

### Epic 6: See How My World Connects

**Stories:** US-028, US-029, US-030, US-031.

**Built:** Graph storage and repository methods for entities, relationships, and neighbors.

**Remaining:**

- Add authorized graph APIs for browsing, search, details, and bounded neighborhood loading.
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

**Supports:** Epics 3, 4, and 5.

The researcher prompt and initial runtime tools are implemented.
The old extraction pipeline is removed. Source references and unresolved design decisions still need completion.

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
5. Expose research status and safe retry through the API and client.
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

1. Expose scoped graph browsing and detail APIs.
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

1. Fix account errors and settle world-name validation.
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

## Immediate next piece

Verify the researcher against the existing prompt examples.
Check saved facts, explicit corrections, unresolved conflicts, and ambiguous identity through the new runtime.
Complete source references and durable jobs without reintroducing name-based write targets.

This is the next small piece. The full researcher replacement is not one task.
