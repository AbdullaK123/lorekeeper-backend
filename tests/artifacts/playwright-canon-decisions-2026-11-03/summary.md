# Canon authority regression — 2026-11-03

**Result: all 10 browser steps pass.** Five paid chat turns run through real Chromium against the rebuilt local API and isolated QA data.

World: `01a1a05a-c533-7920-96a4-49528ccb633e`.
Original conversation: `01a1a05a-c5ac-71d2-9bef-32c15f00a40b`.
Mara entity: `142e7fb6-c6b5-4758-9c59-9806b77862c4`.

## Verified behavior

1. Signup, world creation, and conversation creation work.
2. Research saves four entities and four relationships from stated lore.
3. Graph/source inspection and the original age-34 answer work.
4. A disputed age-41 note creates a conflict without changing age 34.
5. Accept-new resolves the conflict and saves age 41.
6. A later question in the same conversation answers `41`.
7. Completed research leaves the entity, properties, and source IDs exactly equal to the post-acceptance snapshot.
8. A fresh conversation also answers `41`; its research leaves the entity unchanged.
9. Completed-job retry displays 409; an invalid entity UUID displays plain-text 400.
10. No uncaught browser page errors appear.

[Question workflow evidence](question-tools.json) shows all three question-only research workflows have no tool calls, including no graph mutations.
[Conversation evidence](conversations.json) preserves both threads. [The raw browser report](report.json) records assertions and timing.

## Changes under test

- Research input uses user_message, matching its prompt.
- Prompts distinguish current canon from old history and require no writes for question-only messages.
- Current retrieved lore is supplied as system data rather than inserted into the first old user message.
- Retrieval includes the writer's resolved conflict decisions, making the out-of-band acceptance visible to chat.

The earlier [input/prompt-only run](../playwright-canon-fix-2026-11-03/report.json) preserves storage but still answers 34.
The [system-context-only run](../playwright-canon-context-2026-11-03/report.json) also still answers 34.
Its text/vector diagnostic confirms retrieval returns property age 41. Adding the actual writer decision completes this tested correction.

## Limits

The description still says age 34. This run deliberately verifies property/decision authority despite that stale summary.
It does not repair summary text or prove every conflict target and phrasing.
This is one passing scenario, not a guarantee that prompt rules make all writes safe.
The final source is tested through a rebuilt debug API. The existing Docker image is not rebuilt and needs a new build to include these changes.
The test API is stopped. QA world data and screenshots remain for inspection.
