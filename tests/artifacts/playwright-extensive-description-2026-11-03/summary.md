# Extensive description acceptance test — 2026-11-03

**Result: 28 of 28 browser steps pass in 192.75 seconds.**

The run uses real Chromium, a rebuilt local debug API, and new synthetic QA data.
It sends 20 paid chat turns with background research, plus two description-rewrite requests.
The second rewrite occurs during repeated acceptance, although the repository preserves the first accepted result.

World: `01a1a113-dace-70af-8619-6be9d3f6b120`.
Original conversation: `01a1a113-db28-7b97-bc6f-efa3df827df1`.
Fresh conversation: `01a1a114-e845-7e10-ab1b-03e50ff87cfd`.

## Checks

- Signup, world/thread creation, lore storage, graph reads, and message source inspection pass.
- A disputed age creates a conflict while the existing age remains unchanged.
- Acceptance saves property age 41 and rewrites the description to age 41.
- The description retains Bellhaven and Glasswatch and contains no old age 34.
- Repeated acceptance preserves the full entity and the resolved decision remains visible.
- Existing and fresh conversations answer 41; completed research preserves entity data and source IDs.
- Fifteen more questions retain age 41 through a longer conversation (32 messages in the fresh thread).
- Every extended question waits for completed research and compares the entire entity with the accepted snapshot.
- Page reload preserves the saved conversation exactly.
- At 390×844, the workbench reads the same entity and keeps its action visible.
- Completed-job retry returns 409; invalid entity ID returns 400. Both errors appear in the UI.
- No uncaught browser page errors occur.

Saved description: `Keeper of Glasswatch Lighthouse; born in Bellhaven; 41 years old.`

[Raw report](report.json), [accepted entity](accepted-entity.json),
[post-question entity](entity-after-question.json), and [extended conversation](extended-conversation.json)
retain the assertions and data. Screenshots cover all 28 steps.

## Limits

This verifies one entity-property correction with several question forms, not every conflict target.
It does not test concurrent edits, crash recovery, deletion races, multiple browser engines, or production hosting.
The free repository regression passed previously; it is not rerun here. No full suite rerun occurs.
The Docker image is not rebuilt. The temporary API stops after this run; QA records remain.
The test checks unchanged graph data, not the absence of all research tool calls.
