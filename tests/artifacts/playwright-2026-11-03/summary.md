# Playwright writer workflow — 2026-11-03

**Result: failed canon-preservation acceptance.** The UI can accept a conflict, but later chat/research reverts that decision.

Used real Chromium against the built playground and API image at localhost:18000.
Data lives in the isolated lorekeeper_adversarial database. Five real chat turns trigger paid research/model calls.
No production source changes are made during this test.

## Scenario and evidence

World: `01a1a039-6863-7327-b745-793905a85caa`.
Original conversation: `01a1a039-68b6-7d0e-8ff2-baa4fcd919fc`.
Entity Mara Vale: `64c27749-f9a3-4fc2-9348-e3e8419be0c5`.

1. Signup, world creation, and conversation creation pass through the browser UI.
2. State that Mara is 34, born in Bellhaven, and keeps Glasswatch Lighthouse.
3. Research completes. The graph contains four entities and three relationships. Source lookup returns the original statement.
4. Ask about saved lore. The reply correctly gives age 34 and birthplace Bellhaven.
5. Report a disputed note saying 41, explicitly retaining 34 until review. An age conflict appears; saved age remains 34.
6. Choose accept-new in the workbench. The conflict resolves and the entity property becomes 41.
7. The description still says `age 34`. See [accepted entity](accepted-entity.json).
8. Ask: “How old is Mara Vale now, according to the accepted canon? Give her current age only.”
9. The reply says `34`. See [original conversation](original-conversation.json) and [browser screenshot](07-retrieve-accepted-canon.png).
10. Research for that question calls update_graph with age 34, replacing the accepted property value.
    Its source list includes the question's message ID. See [sanitized workflow checkpoints](question-workflow.json).
11. A browser read confirms saved age is back to 34. See [reverted entity](reverted-entity.json) and [screenshot](10-reverted-canon.png).
12. A fresh conversation also answers 34 after the database reversion. This does not isolate fresh retrieval as a separate defect.

The question's message ID is `ea1e5a44-23f9-4be7-a05f-ea733a1326b3`.
The accepting operation succeeds; the later question and research undo its meaning and stored value.
Stale description, old conversation context, and missing decision authority are inputs to investigate; this run does not isolate one root cause.

## Other checks

- Completed-job retry returns 409 and displays its error.
- An invalid entity UUID returns 400. The workbench displays the plain-text backend error.
- World selection and workbench controls remain visible at 390×844.
- No uncaught browser page errors are recorded.
- Eligible failed-job recovery, interrupted streams, concurrent turns, and full mobile accessibility are not covered.

Harness setup initially needs an installed Chromium path and broader label selectors.
These are test harness corrections, not application fixes. Resumed runs reuse the same account/world and saved turns.
The raw [workflow report](workflow-report.json) and [follow-up report](report.json) retain the failing assertions.
Encrypted provider reasoning is omitted from the saved checkpoint evidence; only tool calls/results remain.

The temporary API container is removed after testing. The two project database services remain running with their existing volumes.
Synthetic world data stays in QA for inspection. No token/cost total is inferred from partial UI usage figures.
