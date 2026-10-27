# Lorekeeper — User Stories

These stories serve writers with large fictional worlds and lore spread across notes, documents, and conversations.
They need to remember details, avoid contradictions, and understand how their worlds connect.

The writer owns the canon: facts they establish or confirm about their fictional world.
These stories describe intended product behavior, not current implementation status.

## Epic 1: Keep My Worlds Private and Separate

- **US-001:** As a writer, I want a private account so I can protect my unpublished lore.
- **US-002:** As a writer, I want separate worlds so I can develop different settings without mixing their lore.
- **US-003:** As a writer, I want to name and describe each world so I can recognize it later.
- **US-004:** As a writer, I want world deletion to remove its conversations and lore so I can discard a setting completely.

## Epic 2: Build My World Through Conversation

- **US-005:** As a writer, I want to describe my world naturally so I can develop ideas without filling out forms.
- **US-006:** As a writer, I want to paste existing notes into chat so I can gather scattered lore.
- **US-007:** As a writer, I want separate conversations within a world so I can explore different topics.
- **US-008:** As a writer, I want to resume earlier conversations so I can continue where I left off.
- **US-009:** As a writer, I want relevant follow-up questions so I can clarify the details that matter.

## Epic 3: Remember My Lore Without Manual Filing

- **US-010:** As a writer, I want stated lore saved automatically so I can focus on thinking aloud.
- **US-011:** As a writer, I want minor details remembered so I can recover them when they matter later.
- **US-012:** As a writer, I want alternate names recognized through context so I can refer to characters naturally.
- **US-013:** As a writer, I want new details added without losing earlier facts so I can trust my world's memory.
- **US-014:** As a writer, I want repeated mentions combined so I can avoid duplicate records for the same subject.
- **US-015:** As a writer, I want save failures made visible so I can retry without silently losing lore.

## Epic 4: Find Answers I Can Trust

- **US-016:** As a writer, I want to ask questions in ordinary language so I can find lore without remembering exact wording.
- **US-017:** As a writer, I want answers drawn from my saved lore so I can rely on my own canon.
- **US-018:** As a writer, I want answers linked to my earlier statements so I can verify their source.
- **US-019:** As a writer, I want missing information stated clearly so I can distinguish unknown details from established facts.
- **US-020:** As a writer, I want relationship conditions preserved in answers so I can understand when a connection applies.

## Epic 5: Keep Control of Canon and Contradictions

- **US-021:** As a writer, I want contradictions flagged against earlier lore so I can catch mistakes before they spread.
- **US-022:** As a writer, I want contradictions across connected facts identified so I can catch problems beyond one character's description.
- **US-023:** As a writer, I want both disputed claims and their sources shown so I can decide what remains canon.
- **US-024:** As a writer, I want unresolved contradictions kept visible so I can return to them without losing either claim.
- **US-025:** As a writer, I want explicit corrections applied so I can change my world deliberately.
- **US-026:** As a writer, I want suggestions and speculation kept outside canon so I can brainstorm without committing ideas.
- **US-027:** As a writer, I want unclear references surfaced so I can clarify them before facts attach to the wrong character.

## Epic 6: See How My World Connects

- **US-028:** As a writer, I want a visual map of my world's connections so I can see its overall shape.
- **US-029:** As a writer, I want to inspect a character's connections so I can understand their place in the world.
- **US-030:** As a writer, I want to follow connections between people, places, factions, and events so I can trace related lore.
- **US-031:** As a writer, I want connection details and direction shown so I can understand who affects whom.

## Epic 7: Reason About Consequences Within My World

- **US-032:** As a writer, I want to explore possible changes so I can understand their effects before changing canon.
- **US-033:** As a writer, I want consequences traced through established relationships so I can judge whether the reasoning fits my world.
- **US-034:** As a writer, I want inferred consequences distinguished from saved facts so I can evaluate them before accepting them.
- **US-035:** As a writer, I want missing premises identified so I can see where my world's rules need more detail.

## Epic 8: Find Gaps and Choose What to Develop

- **US-036:** As a writer, I want undeveloped parts of my world identified so I can choose where to focus next.
- **US-037:** As a writer, I want related factions compared so I can notice uneven detail across their cultures or institutions.
- **US-038:** As a writer, I want unanswered questions gathered so I can return to unfinished worldbuilding.
- **US-039:** As a writer, I want gaps presented without invented answers so I can keep creative control.

The following epics cover SaaS accounts, payments, communication, and operation.
Operator stories support the writer-facing service.

## Epic 9: Access and Recover My Account

- **US-040:** As a writer, I want to sign in through a supported OAuth provider so I can access my account without another password.
- **US-041:** As a writer, I want to verify my email address so I can receive account messages at an address I control.
- **US-042:** As a writer, I want to reset a forgotten password securely so I can recover access to my worlds.
- **US-043:** As a writer, I want to manage linked sign-in methods safely so I can keep access without creating duplicate accounts.
- **US-044:** As a writer, I want to receive clear account recovery errors so I can understand my next safe step.

## Epic 10: Manage My Subscription and Payments

- **US-045:** As a writer, I want to see plan prices and included limits so I can choose a suitable subscription.
- **US-046:** As a writer, I want to subscribe through Stripe checkout so I can pay securely.
- **US-047:** As a writer, I want to view invoices and update payment details so I can manage my billing.
- **US-048:** As a writer, I want to change or cancel my subscription so I can control future charges.
- **US-049:** As a writer, I want to see payment failures and resulting access changes so I can restore service without surprises.
- **US-050:** As a operator, I want to reconcile billing events with account access so I can avoid granting or removing access incorrectly.

## Epic 11: Understand and Control Usage

- **US-051:** As a writer, I want to see my usage and remaining allowance so I can understand my available service.
- **US-052:** As a writer, I want to see clear limits before expensive work starts so I can avoid unexpected interruptions or charges.
- **US-053:** As a writer, I want to understand when allowances renew so I can plan my work.
- **US-054:** As a operator, I want to enforce model spending and abuse limits so I can keep service costs bounded.

## Epic 12: Receive Essential Service Emails

- **US-055:** As a writer, I want to receive verification and recovery emails so I can complete essential account actions.
- **US-056:** As a writer, I want to receive important billing and service notices so I can respond to changes affecting my account.
- **US-057:** As a operator, I want to track failed email delivery and retry safely so I can keep essential account messages reliable.

## Epic 13: Operate and Recover the Service

- **US-058:** As a operator, I want to detect failures through logs, metrics, and alerts so I can restore service quickly.
- **US-059:** As a operator, I want to restore tested backups so I can recover from data loss.
- **US-060:** As a operator, I want to deploy and roll back compatible changes so I can release without losing accepted work.
- **US-061:** As a writer, I want to see useful outage and recovery information so I can understand service availability.

## Epic 14: Protect and Manage My Data

- **US-062:** As a writer, I want to export my worlds and source material so I can keep an independent copy.
- **US-063:** As a writer, I want to delete my account and its data so I can leave the service deliberately.
- **US-064:** As a writer, I want to understand retention and external AI processing so I can decide whether to trust the service.
- **US-065:** As a operator, I want to limit abusive requests and protect credentials so I can reduce unauthorized access and service disruption.
- **US-066:** As a operator, I want to restrict and record privileged access so I can investigate issues without unrestricted access to private lore.

## Epic 15: Get Help and Resolve Service Problems

- **US-067:** As a writer, I want to find a support contact and report a problem so I can get help when workflows fail.
- **US-068:** As a writer, I want to identify a failed operation without sharing private lore so I can give support useful evidence.
- **US-069:** As a operator, I want to inspect and retry failed operations with controlled tools so I can resolve incidents safely.
