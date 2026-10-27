# Lorekeeper — Design and Development Lifecycle

Updated: 2026-10-27.

The [user stories](user_stories.md) define the required outcomes.
The [product vision](product_soul.md) defines the audience and purpose.
The [project plan](project_plan.md) tracks current work and delivery order.
This document describes each epic through six lifecycle stages.

Design choices below describe intended behavior. They do not claim completed implementation.
Open decisions require answers before the affected build work starts.

Each epic follows this cycle:

1. **Define:** State the writer's need, scope, and acceptance conditions.
2. **Design:** Describe data, system boundaries, user flow, and failure behavior.
3. **Build:** Deliver small pieces using existing foundations where possible.
4. **Test:** Verify behavior, failure recovery, privacy, and writer acceptance.
5. **Deploy:** Release compatible changes and verify the deployed workflow.
6. **Maintain:** Monitor behavior, fix regressions, and repeat the cycle when needs change.

The project reaches delivery completion when all epics pass acceptance in the deployed product.
Maintenance continues after delivery completion.

## Epic 1: Keep My Worlds Private and Separate

### 1. Define

**Stories:** US-001, US-002, US-003, US-004.

Writers need private accounts and separate worlds for unpublished lore.
They can name, describe, and delete a world.
Deletion must remove conversations and graph data, including data created by background work.

**Acceptance:** No supported path exposes another writer's data.
A deleted world stays deleted after retries and restarts.

### 2. Design

- Authenticate requests through account sessions.
- Check world ownership in every API and tool before accessing its data.
- Derive tool scope from trusted request or job context, not model-supplied ownership fields.
- Keep world deletion and cleanup job creation in one Postgres transaction.
- Retry graph cleanup safely after partial failure.
- Coordinate research writes with deletion so stale jobs cannot restore removed lore.
- Show account errors, world validation errors, and pending deletion in the client.

**Open decisions:** World-name rules, deletion status visibility, and coordination between research and deletion.

### 3. Build

1. Reuse account, session, world, and cleanup foundations.
2. Fix duplicate-signup errors and agreed world validation rules.
3. Add missing ownership checks as new tools and routes appear.
4. Complete deletion coordination and account/world client flows.

### 4. Test

- Try foreign world IDs through every route and tool.
- Test expired sessions, logout, duplicate signup, and invalid world input.
- Delete a world during active research and during cleanup failure.
- Restart workers and verify complete deletion without affecting other worlds.

### 5. Deploy

- Apply compatible schema changes before dependent application changes.
- Verify session configuration and access controls in the deployed environment.
- Exercise signup, login, world updates, and deletion using isolated test accounts.
- Confirm cleanup runs and reports failures without exposing private content.

### 6. Maintain

- Monitor account failures, denied access, cleanup failures, and old cleanup jobs.
- Review ownership checks whenever a route or tool changes.
- Verify backup access controls and recovery procedures.
- Investigate any recreated data after deletion as a correctness defect.

## Epic 2: Build My World Through Conversation

### 1. Define

**Stories:** US-005, US-006, US-007, US-008, US-009.

Writers describe lore naturally, paste notes, use separate conversations, and resume earlier discussions.
The conversation asks relevant questions without inventing canon.

**Acceptance:** Saved turns survive leaving and returning.
Interruptions and retries do not silently lose or duplicate accepted turns.

### 2. Design

- Keep the conversation service responsible for chat flow and message storage.
- Keep lore tools read-only during conversational replies.
- Define stable message references for sources and retries.
- Define stream events for partial text, saved completion, and failure.
- Show reply progress separately from background lore-save progress.
- Set note-size and history limits with clear client feedback.
- Define context selection for conversations larger than the model's input limit.

**Open decisions:** Concurrent turn ordering, interrupted reply storage, retry identity, and supported note size.

### 3. Build

1. Reuse conversation routes, streamed replies, and atomic turn storage.
2. Add stable message references and the agreed stream contract.
3. Complete chat navigation, note pasting, resume, and retry states.
4. Add long-history handling and improve follow-up behavior through evaluated examples.

### 4. Test

- Paste notes, exchange turns, leave, and resume.
- Interrupt streams before and after message storage.
- Retry the same request and send competing turns.
- Test input limits, long histories, and relevant follow-up questions.

### 5. Deploy

- Release stream changes with a compatible client.
- Verify streaming through the deployed proxy and its timeout settings.
- Check stored history and retry behavior using an isolated conversation.
- Verify that reply completion does not falsely report completed lore saving.

### 6. Maintain

- Monitor reply failures, response delay, disconnects, and message-storage failures.
- Review context quality as conversations grow.
- Keep examples of poor follow-up questions for regression checks.
- Update client handling whenever the stream contract changes.

## Epic 3: Remember My Lore Without Manual Filing

### 1. Define

**Stories:** US-010, US-011, US-012, US-013, US-014, US-015.

The system saves stated lore without manual filing.
Updates preserve earlier facts, minor details, and subject identity.
Failures remain visible and retryable.

**Acceptance:** Repeated processing and restarts neither lose accepted facts nor create duplicate subjects.
Unclear references never silently attach facts to the wrong subject.

### 2. Design

- Use the [researcher prompt](../prompts/researcher.md) to guide search, reading, interpretation, and updates.
- Keep Memgraph as the graph store during the replacement.
- Use exact saved entity references for writes and relationship endpoints.
- Preserve source references, prior facts, conditions, and unresolved conflicts during updates.
- Persist research jobs with saved turns in one Postgres transaction.
- Bound job work and preserve status across retries and restarts.
- Prevent competing jobs from overwriting newer accepted facts.
- Surface pending work, failures, and unresolved references to the writer.

**Open decisions:** Tool schemas, same-name entity storage, source representation, write coordination, and recovery after partial graph writes.

### 3. Build

1. Define exact references and search/full-read tool results.
2. Build read tools, then entity writes, relationship writes, and conflict tools.
3. Verify one saved statement through graph read-back before widening behavior.
4. Add durable jobs, safe retries, status APIs, and client feedback.
5. Replace old extraction only after the researcher passes behavioral checks.

### 4. Test

- Test shared aliases, canonical-name collisions, and distinct subjects with identical names.
- Preserve minor facts and relationship conditions through repeated updates.
- Fail between graph writes, restart, and retry the job.
- Run competing jobs and delete the world during research.
- Verify that failed work stays visible and retries do not duplicate facts.

### 5. Deploy

- Introduce job and source storage before switching processing behavior.
- Evaluate the researcher in isolated worlds before enabling production writes.
- Keep one production writer pipeline active during the switch.
- Define how pending jobs continue or pause during rollback.
- Verify saved facts and job status after deployment.

### 6. Maintain

- Monitor job age, failures, retries, processing delay, and model cost.
- Track wrong-subject writes, lost facts, and duplicate records as correctness defects.
- Version prompts and tool contracts together.
- Recheck representative cases after model, prompt, or tool changes.

## Epic 4: Find Answers I Can Trust

### 1. Define

**Stories:** US-016, US-017, US-018, US-019, US-020.

Writers ask ordinary-language questions and receive answers grounded in saved lore.
Answers preserve conditions and link claims to earlier statements.
Missing information stays explicit.

**Acceptance:** The writer can inspect the evidence for an answer.
The system does not present missing or disputed information as established fact.

### 2. Design

- Combine text search and vector search, which finds related meaning.
- Expand relevant results through saved graph connections.
- Preserve entity properties, neighbor properties, relationship direction, and conditions in answer context.
- Carry stable source references through storage, retrieval, and client links.
- Distinguish established facts, disputed claims, and unknown details.
- Show when relevant background research remains pending.

**Open decisions:** Fact-level source storage, citation format, retrieval limits, and source behavior after conversation deletion.

### 3. Build

1. Fix missing properties and relationship descriptions in retrieval output.
2. Verify ranking behavior when search results repeat.
3. Add source references to stored lore and retrieved results.
4. Add answer source links and source inspection in the client.
5. Evaluate unknown and disputed-answer behavior.

### 4. Test

- Ask indirectly worded questions about minor details.
- Test misleading search matches and repeated search rows.
- Verify each answer link reaches the supporting statement.
- Test time limits, conditional relationships, unknown facts, and disputed claims.
- Confirm retrieval never crosses world boundaries.

### 5. Deploy

- Verify search indexes and source storage before enabling citation output.
- Define how existing lore without sources appears to writers.
- Run a deployed question-and-source check against an isolated world.
- Keep answer formats compatible with the deployed client.

### 6. Maintain

- Monitor retrieval failures, answer delay, and broken source links.
- Collect unsupported-answer cases without logging private lore by default.
- Recheck answer quality after search, embedding, prompt, or model changes.
- Repair missing sources without inventing historical evidence.

## Epic 5: Keep Control of Canon and Contradictions

### 1. Define

**Stories:** US-021, US-022, US-023, US-024, US-025, US-026, US-027.

Canon means facts the writer establishes or confirms.
Writers inspect contradictions, clarify references, and make deliberate corrections.
Suggestions and hypothetical statements remain outside canon until confirmed.

**Acceptance:** Both disputed claims remain available until the writer decides.
The decision updates affected lore without erasing unrelated facts.

### 2. Design

- Store disputed claims, their sources, affected subjects, and resolution state.
- Check relevant connected facts as well as one entity's fields.
- Keep unresolved claims separate from accepted facts.
- Provide conflict review, clarification, and explicit correction flows.
- Preserve the writer's decision and its source for later review.
- Reject stale resolution requests that would overwrite a newer decision.

**Open decisions:** Resolution actions, clarification state, and consistent updates across affected facts and relationships.

### 3. Build

1. Reuse conflict storage and existing contradiction examples.
2. Add source-aware conflict tools and unresolved-reference results.
3. Add authorized listing, inspection, and resolution APIs.
4. Build client review and clarification flows.
5. Connect explicit corrections to safe lore updates.

### 4. Test

- Test direct contradictions and contradictions across connected facts.
- Distinguish a correction from an unresolved disagreement.
- Reject unapproved assistant suggestions as canon.
- Keep hypothetical statements outside canon.
- Resolve competing decisions safely and preserve unrelated facts.

### 5. Deploy

- Preserve existing unresolved conflicts during schema changes.
- Enable review only when sources and resolution writes work together.
- Verify an entire conflict review in the deployed client.
- Confirm rollback preserves recorded writer decisions.

### 6. Maintain

- Monitor failed resolutions and old unresolved issues.
- Review false conflict reports and missed contradictions.
- Keep regression cases for corrections, speculation, and ambiguous references.
- Preserve decision history when prompts or storage change.

## Epic 6: See How My World Connects

### 1. Define

**Stories:** US-028, US-029, US-030, US-031.

Writers explore a visual graph, inspect subjects, and follow their connections.
Connection details explain direction and conditions.

**Acceptance:** A writer can trace connections across characters, places, factions, and events.
Large worlds remain usable without loading the entire graph at once.

### 2. Design

- Add authorized graph browsing, search, detail, and neighborhood APIs.
- Bound result sizes and support loading more connections.
- Use stable entity and relationship IDs across API responses and client selection.
- Show connection direction, conditions, details, and source links.
- Provide clear empty, loading, failure, and partial-result states.
- Provide an accessible list view alongside the visual graph.

**Open decisions:** Graph response contracts, navigation limits, layout approach, and supported world size.

### 3. Build

1. Reuse graph repository reads behind scoped services and routes.
2. Build graph navigation and entity inspection views.
3. Add connection details and source navigation.
4. Add bounded loading, search, and accessible navigation.

### 4. Test

- Verify owner checks and world boundaries on every graph read.
- Follow directed and conditional connections across entity types.
- Test empty worlds, disconnected subjects, repeated loading, and deleted subjects.
- Check large-world response times and client interaction.
- Verify keyboard navigation and readable connection details.

### 5. Deploy

- Release graph APIs before dependent client views.
- Verify query limits and required indexes.
- Browse a representative isolated world in the deployed client.
- Confirm partial results do not imply missing connections are absent from canon.

### 6. Maintain

- Monitor slow graph queries, oversized responses, and client rendering failures.
- Check navigation after graph schema changes.
- Keep performance examples for dense and disconnected graphs.
- Preserve accessible navigation as visual behavior changes.

## Epic 7: Reason About Consequences Within My World

### 1. Define

**Stories:** US-032, US-033, US-034, US-035.

Writers explore possible changes without changing canon.
The system traces consequences through established facts and identifies missing premises.
A premise is a fact needed to support a conclusion.

**Acceptance:** Answers separate saved facts, inferred consequences, and unknown details.
Exploration changes no canon without explicit writer confirmation.

### 2. Design

- Mark hypothetical context explicitly for both chat and background research.
- Retrieve relevant relationship chains with sources and conditions.
- Explain each inferred consequence through supporting facts.
- Stop unsupported conclusions where required premises are missing.
- Offer deliberate confirmation before a proposed change becomes canon.
- Bound graph traversal and report incomplete evidence.

**Open decisions:** Hypothetical context boundaries, evidence path format, traversal limits, and confirmation behavior.

### 3. Build

1. Establish hypothetical handling across chat and research.
2. Add retrieval for relevant chains of relationships.
3. Add consequence explanations, source links, and missing-premise output.
4. Add explicit confirmation and safe saving through the canon workflow.

### 4. Test

- Explore a grain export ban using saved trade dependencies.
- Remove a required premise and verify the answer reports the gap.
- Test cycles, conflicting evidence, and conditional relationships.
- Verify graph contents stay unchanged during hypothetical discussion.
- Confirm only approved changes enter canon.

### 5. Deploy

- Enable reasoning after grounded retrieval and hypothetical safeguards pass.
- Run a deployed scenario and compare graph contents before and after discussion.
- Confirm evidence links work in the client.
- Preserve read-only exploration if confirmation writes fail.

### 6. Maintain

- Track unsupported conclusions and accidental canon writes.
- Recheck reasoning cases after model or retrieval changes.
- Review traversal limits as worlds grow.
- Keep missing-premise reporting useful rather than producing forced answers.

## Epic 8: Find Gaps and Choose What to Develop

### 1. Define

**Stories:** US-036, US-037, US-038, US-039.

Writers compare saved lore, notice uneven detail, and gather open questions.
They decide which gaps matter.
The system does not invent answers or impose a universal worldbuilding checklist.

**Acceptance:** Gap reports cite their basis and keep creative control with the writer.
Open questions remain available across sessions.

### 2. Design

- Compare related subjects using their saved details.
- Distinguish absent evidence from a confirmed absence in the fictional world.
- Present uneven detail as a question, not a contradiction or required correction.
- Save open questions with world scope and links to relevant subjects.
- Let writers revisit, answer, or dismiss questions.
- Keep generated questions outside canon.

**Open decisions:** Comparison scope, question states, duplicate question handling, and refresh behavior after lore changes.

### 3. Build

1. Add grounded comparisons using source-aware retrieval.
2. Add gap reports with evidence and clear limits.
3. Add open-question storage, APIs, and client views.
4. Connect answered questions to confirmed lore through existing save flows.

### 4. Test

- Compare detailed and sparse factions without inventing shared institutions.
- Distinguish unknown military structure from an explicitly absent military.
- Save, revisit, answer, and dismiss questions.
- Update lore and verify old questions do not become misleading.
- Verify reports and saved questions remain private to their world.

### 5. Deploy

- Apply question storage changes before enabling persistent review.
- Verify a comparison and saved-question workflow in an isolated deployed world.
- Confirm failed report generation leaves existing questions intact.
- Verify that question generation writes no canon facts.

### 6. Maintain

- Monitor failed reports, repeated questions, and outdated questions.
- Review whether comparisons help writers choose their next work.
- Recheck reports after prompt and model changes.
- Preserve writer dismissals and decisions when refreshing reports.

## Epic 9: Access and Recover My Account

### 1. Define

**Stories:** US-040, US-041, US-042, US-043, US-044.

Writers need secure sign-in and recovery without losing their existing worlds.

**Acceptance:** A writer can sign in, recover access, and link identities without exposing or duplicating their worlds.

### 2. Design

- Keep password sessions and add OAuth, which lets an external identity provider confirm identity.
- Use verified provider identity IDs; never merge accounts from matching email alone.
- Require authenticated confirmation for account linking and protect OAuth redirects against forged requests.
- Use short-lived, single-use verification and recovery tokens; avoid exposing whether an email has an account.

**Open decisions:** Provider selection, account linking rules, token expiry, and recovery for accounts without passwords.

### 3. Build

- Extend the existing account model with verified identities and recovery records.
- Build verification, OAuth, linking, and recovery flows with client feedback.

### 4. Test

- Test forged redirects, reused tokens, expired tokens, and duplicate provider identities.
- Verify account linking cannot take over another account or remove its last usable sign-in method.

### 5. Deploy

- Configure provider credentials and exact deployed callback addresses.
- Verify each supported sign-in and recovery flow using isolated accounts.

### 6. Maintain

- Monitor sign-in and recovery failures without recording tokens.
- Rotate credentials and review provider changes.

## Epic 10: Manage My Subscription and Payments

### 1. Define

**Stories:** US-045, US-046, US-047, US-048, US-049, US-050.

Writers need clear pricing, secure payment, and predictable subscription changes.

**Acceptance:** Payment state reliably controls access, and writers can manage billing without losing their lore.

### 2. Design

- Use Stripe-hosted checkout and billing management; do not store payment card details.
- Verify signed billing events and persist received events before processing.
- Handle duplicate and out-of-order events safely; reconcile local state with Stripe.
- Keep subscription state separate from feature access rules.
- Preserve lore when payment fails; define read access, grace periods, and cancellation timing.

**Open decisions:** Prices, currencies, taxes, refunds, trial policy, plan changes, and payment-failure access rules.

### 3. Build

- Add customer, subscription, billing-event, and access records.
- Build checkout, billing management, event processing, reconciliation, and plan screens.

### 4. Test

- Test duplicate events, delayed events, failed payments, cancellation, and plan changes in Stripe test mode.
- Verify browser checkout redirects alone cannot grant paid access.

### 5. Deploy

- Verify test-mode workflows before enabling live credentials and event delivery.
- Publish prices and billing terms; verify reconciliation and rollback procedures.

### 6. Maintain

- Monitor event backlog, failed payments, and access mismatches.
- Reconcile subscriptions periodically and investigate charge disputes through controlled support access.

## Epic 11: Understand and Control Usage

### 1. Define

**Stories:** US-051, US-052, US-053, US-054.

Writers need predictable allowances. Operators need enforceable spending controls.

**Acceptance:** Concurrent chat and background jobs respect limits, and writers see accurate usage and renewal information.

### 2. Design

- Record model usage by account, world, operation, and processing attempt.
- Reserve allowance before costly work and settle it against recorded usage.
- Make reservations safe across concurrent requests, retries, failures, and worker restarts.
- Include background research, embeddings, and retries in cost controls.
- Separate customer allowances from internal provider-cost budgets; never charge twice for the same billable action.

**Open decisions:** Allowance units, reset periods, retry charging, queued-work policy, and hard spending ceilings.

### 3. Build

- Build usage records, reservations, access checks, and worker budget enforcement.
- Add usage screens, renewal information, and clear limit responses.

### 4. Test

- Race concurrent requests against the final available allowance.
- Test failed calls, retries, expired reservations, renewal boundaries, and plan changes.

### 5. Deploy

- Validate accounting with isolated workloads before enforcing customer limits.
- Set internal spending ceilings and verify workers obey them.

### 6. Maintain

- Reconcile recorded usage and provider costs.
- Alert on abnormal spending, stuck reservations, and repeated limit bypass attempts.

## Epic 12: Receive Essential Service Emails

### 1. Define

**Stories:** US-055, US-056, US-057.

Essential emails help writers access accounts and understand service changes.

**Acceptance:** Essential messages arrive reliably, and failures remain visible and recoverable.

### 2. Design

- Use a service email provider and a durable outbound queue.
- Keep email content minimal; never include private lore or reusable credentials.
- Use message identity to prevent duplicate sends during retries.
- Track delivery failures, bounces, and suppressed addresses.
- Avoid duplicating billing notices already sent by Stripe.

**Open decisions:** Email provider, sender domain, required notices, and which system owns each billing notice.

### 3. Build

- Build templates, queued delivery, provider integration, and delivery status handling.
- Connect account and billing events to the required messages.

### 4. Test

- Test provider outages, duplicate processing, rejected addresses, and expired recovery links.
- Verify templates, links, sender identity, and private-content exclusion.

### 5. Deploy

- Verify sender-domain configuration and test real delivery to controlled addresses.
- Enable failure alerts and documented manual recovery.

### 6. Maintain

- Monitor queue age, bounces, complaints, and failed delivery.
- Review template accuracy and sender reputation.

## Epic 13: Operate and Recover the Service

### 1. Define

**Stories:** US-058, US-059, US-060, US-061.

The deployed service needs observable failures, tested recovery, and safe releases.

**Acceptance:** Operators detect failures and prove recovery within agreed downtime and data-loss targets.

### 2. Design

- Measure API, database, graph, worker, email, billing, and model-provider health.
- Keep private lore, credentials, and tokens out of routine logs.
- Define backup schedules and consistent recovery across Postgres and Memgraph.
- Define shutdown, migration, job recovery, readiness checks, and rollback behavior.

**Open decisions:** Hosting, alert ownership, acceptable downtime, acceptable data loss, and backup retention.

### 3. Build

- Add health checks, structured failure reporting, alerts, and release automation.
- Write incident, restore, migration, and rollback procedures.

### 4. Test

- Restore both stores into an isolated environment and verify source links and pending jobs.
- Test deployment shutdown, dependency outages, restart recovery, and rollback compatibility.

### 5. Deploy

- Prove restore and rollback procedures before accepting paid customers.
- Assign an alert recipient and verify deployed checks and notifications.

### 6. Maintain

- Run restore drills and review alert usefulness.
- Track capacity, dependency updates, and incidents with corrective actions.

## Epic 14: Protect and Manage My Data

### 1. Define

**Stories:** US-062, US-063, US-064, US-065, US-066.

Writers own private worlds and need clear data controls and processing disclosures.

**Acceptance:** Writers can export and delete data, and published privacy behavior matches deployed handling.

### 2. Design

- Define export content, source links, format, and secure download expiry.
- Coordinate account deletion with subscriptions, jobs, graph data, identities, and stored exports.
- Document backup expiry and any retained billing records separately from active account data.
- Apply rate limits, secret management, least-privilege access, and dependency review.
- Publish actual data flows, provider processing, retention, and service terms.

**Open decisions:** Export format, retention periods, deletion confirmation, provider data handling, and launch policy requirements.

### 3. Build

- Build export and account-deletion jobs with visible progress and safe retries.
- Add access restrictions, rate limits, administrative audit records, and accurate policy pages.

### 4. Test

- Verify exports contain only the requesting writer’s data.
- Test deletion during active billing and research; verify retries cannot recreate removed data.
- Test abuse limits, privileged access controls, and secret exclusion from logs.

### 5. Deploy

- Review published policies against actual behavior and configured providers.
- Verify account deletion, export delivery, access controls, and secret configuration in the deployed service.

### 6. Maintain

- Review retained data, privileged access, and dependency risks.
- Keep policies current as providers and storage behavior change.

## Epic 15: Get Help and Resolve Service Problems

### 1. Define

**Stories:** US-067, US-068, US-069.

Writers need a support path. Operators need bounded tools to investigate and recover failed work.

**Acceptance:** Writers can report failures, and authorized operators can investigate and recover work without broad data access.

### 2. Design

- Attach safe operation references to errors and support reports.
- Provide authenticated tools for job inspection and approved retries.
- Restrict administrative actions, record their use, and hide private content by default.
- Keep support requests separate from automatic permission to inspect lore.

**Open decisions:** Support channel, response expectations, administrative roles, and rules for authorizing private-content access.

### 3. Build

- Add support links, useful error references, and operational lookup tools.
- Add controlled retry actions and administrative audit records.

### 4. Test

- Verify support references expose no private content.
- Test role restrictions, retry safety, and audit records.

### 5. Deploy

- Publish the support channel and verify report delivery.
- Limit administrative access and rehearse recovery using an isolated failed job.

### 6. Maintain

- Track unresolved reports and recurring failures.
- Convert repeated incidents into regression tests and product fixes.
