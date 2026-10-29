# Fresh-conversation retrieval — 2026-10-29

## Result

**Four checks pass. One demonstrated failure: saved conflicts are invisible to chat.**
Each check uses a separate empty conversation in the existing Ardent Reach world.
No original conversation history is supplied.
All five requests and background research runs complete without reported errors.
The graph remains unchanged after all five questions.

World: `01a18777-7732-7461-8c9f-6545cd1c666e`.
Original worldbuilding conversation: `01a18777-7744-715a-a990-daf20fc93f3f`.
Database: local `lorekeeper_adversarial`.

## Reviewed answers

| Case | Result | Observed behavior |
| --- | --- | --- |
| Minor facts and correction | Pass | Greyhaven birthplace, adult move, jade compass in right boot, Oren's knowledge |
| Relationship conditions | Pass | Council ownership, destruction in 204, no current supply, no bypass |
| Shared title | Pass | Oren guards reservoir and requires permits; Lysa keeps archive with daylight access |
| Unresolved conflicts | Fail | Says no unresolved contradictions despite two saved unresolved conflicts |
| Missing facts | Pass | States population and remaining water are unspecified |

These are manual comparisons against the retained graph and original author statements.
A successful HTTP/test-harness exit does not mean every semantic check passes.

## Demonstrated conflict failure

The writer asks:

> Which contradictions remain unresolved in this world? Give both versions without resolving them. Use only saved lore.

The fresh answer starts:

> I don’t see any unresolved contradictions in the saved lore.

The pre-test graph contains two unresolved conflicts:

1. The aqueduct is permanently destroyed in 204, but a note describes water flowing through it in 206.
2. Civilian archive access is daylight-only, but a note places a civilian scholar there at midnight in 207.

Neither disputed claim reaches the answer.
Instead, the reply explains unrelated apparent tensions about the shared Warden title and historical aqueduct operation.

## Code trace

- `RetrievalService::search` returns entity matches and neighboring entities.
- `get_entity_neighbors` matches `(m:Entity)`, excluding Conflict nodes.
- `SearchHit` has no conflict field; `format_hits` renders no conflicts.
- The chat agent only has `search_world` and cannot call the researcher's full-read tool.

This is an observed retrieval defect, not a hypothetical duplicate-row case.

## Evidence

- [Questions and full answers](transcript.md)
- [Before/after graph and saved messages](report.json)
- Raw SSE responses: one file per named case.

The original world and all five test conversations remain available for inspection.
These five checks do not establish broad retrieval quality or recovery guarantees.
