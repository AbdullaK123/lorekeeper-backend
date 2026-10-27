# Current Sprint

## Sprint 1: Define Researcher Read Tools

**Status:** Planned.
**Start:** 2026-10-27.
**Target review:** 2026-11-03.
**Length:** One week.

A sprint is a short work cycle with one clear goal.
Review and replace this plan every one to four weeks.
Dates guide review; they do not prove completion.

## Goal

Define researcher search and full-read tools before building lore writes.
The contracts must support exact subject selection and traceable source statements.
A contract defines a tool's inputs, outputs, errors, and access rules.

This sprint supports Epic 3, Epic 4, and Epic 5.
It advances US-012, US-013, US-014, US-018, US-023, and US-027.
It does not complete those stories.

## Starting point

- The [researcher prompt](../prompts/researcher.md) exists but is not active in the runtime.
- The current extraction pipeline remains active.
- Graph search and repository reads provide existing foundations.
- Stable source references and distinct same-name subjects need design decisions.

See the [project plan](project_plan.md) for current implementation and recorded test results.

## Committed work

Keep this sprint limited to read contracts and their required identity decisions.
Record the results in [design.md](design.md), under the relevant epics.

| Task | State | Completion evidence |
| --- | --- | --- |
| Define trusted world scope and exact entity references | Planned | Written access rules and accepted/rejected reference examples |
| Decide how distinct subjects with identical names are represented | Planned | Storage decision compatible with exact entity selection |
| Define stable source references | Planned | Message reference format and behavior for existing messages and deleted conversations |
| Define search inputs, results, limits, and errors | Planned | Contract with ambiguous, empty, and foreign-world examples |
| Define full-read inputs, results, limits, and errors | Planned | Contract covering facts, properties, relationships, conditions, conflicts, and sources |
| Walk through prompt examples using the contracts | Planned | Recorded outcomes and remaining gaps |

Use these task states: Planned, In progress, Blocked, Done.
Mark a task Done only when its evidence exists.

## Acceptance checks

- Search can return several possible subjects without choosing one arbitrarily.
- Full-read uses an exact saved entity reference within trusted world scope.
- Shared aliases and canonical-name collisions preserve distinct subjects.
- Results preserve relationship direction, conditions, disputed claims, and available sources.
- Missing sources appear explicitly; the system never invents historical evidence.
- Foreign-world and stale references produce defined errors without leaking private data.
- Result limits clearly indicate incomplete data.
- Every researcher prompt example works through the proposed read contracts.

These are design walkthroughs during this sprint.
Runtime tests follow when the tools are built.

## Outside this sprint

- Researcher write tools and runtime replacement.
- Durable research jobs and deletion coordination.
- Client graph views and conflict review.
- Billing, OAuth, email, and other SaaS implementation.

These remain required in the project plan.
Do not add them here unless the sprint goal changes explicitly.
Urgent privacy or data-loss defects can interrupt the sprint; record that scope change below.

## Dependencies and blockers

- Read the current graph models and queries before choosing contract fields.
- Resolve the unique-name storage constraint before promising support for distinct same-name subjects.
- Resolve source identity before promising links to individual statements.
- Record blockers with the missing decision, affected task, and next action.

**Current blockers:** None recorded. Design decisions remain open within committed work.

## Review and rollover

Review progress on 2026-11-03, or earlier if the goal is met or blocked.
Keep a sprint between one and four weeks.
If extending this sprint, record the reason and revised review date.

At review:

1. Compare each task with its completion evidence.
2. Record completed work and unresolved decisions.
3. Update implementation status and remaining work in the project plan.
4. Archive this document under `docs/sprints/` before replacing its contents.
5. Select the next small goal from the project plan.
6. Recommit unfinished work explicitly; do not carry it forward automatically.

Do not change the wider project scope through sprint rollover.
Update the stories, design, and plan when requirements change.

## Sprint log

| Date | Change or result | Evidence or next action |
| --- | --- | --- |
| 2026-10-27 | Sprint draft created | Define read contracts before write tools |

## Review outcome

**Goal result:** Pending.
**Completed evidence:** Pending.
**Unfinished work and reason:** Pending.
**Next sprint candidate:** Build and verify researcher read tools after contract review.
