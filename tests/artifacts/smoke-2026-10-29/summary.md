# Brinewake smoke test — 2026-10-29

## Result

Completed 20 user turns and 20 assistant replies through the current HTTP/SSE chat endpoint.
All 20 background research runs completed. No stream errors or reported research failures occurred.

Memgraph contains **21 entities, 2 unresolved conflicts, and 36 relationships**.
Two relationships connect entities to conflict nodes; 34 connect lore entities.
The test used a separate account and Postgres database, `lorekeeper_adversarial`.
The synthetic world remains available for inspection. Existing worlds were not changed.

- World ID: `01a1873c-dd93-7dba-a440-e45a4e9563ff`.
- Conversation ID: `01a1873c-dda9-7bf5-a8c5-094ae62ee4c8`.
- Total chat-plus-research time: 627.3 seconds.
- Median turn, including background research: 31.25 seconds.
- Slowest turn: turn 19, 71.8 seconds, replacing a route and retracting old relationships.

## Verified behavior

| Case | Observation |
| --- | --- |
| Linked lore | Guild, fuel, miners, union, ships, people, and locations appear as connected records |
| Minor details | Brass key, left coat cuff, right-eye injury, storm year, black cat, and family relationship survive |
| Explicit birthplace update | Asha is born on Orin, with childhood in Vesper retained; key and guild facts remain |
| Contextual identity | Bren receives the refusal-to-join fact; Asha remains a guild member |
| Contradictions | Sealed-room access and Iven's negotiation restriction produce separate unresolved conflicts |
| Canon preservation | No meeting is added as an accepted event and no invented entrance resolves the dispute |
| Hypothetical discussion | Turn 16 leaves every captured node and relationship unchanged |
| Relationship retraction | Old Nightjar salt-route edges disappear; letter delivery to Kest appears |
| Historical detail | The Nightjar description retains the retired April–September route as history |
| Read-only turns | Turns 8, 12, 15, 16, and 20 leave captured graph records unchanged |
| Final answer | Separates canon, unresolved contradictions, and unknown details |

## Limits and remaining issues

This is one successful smoke run, not a reliability guarantee.
Turns run sequentially, with background research finishing before the next turn.
This does not test concurrent updates, disconnects, process restarts, or deletion races.
Recall answers also have the full conversation history; they do not prove retrieval from a fresh conversation.

Source tracking remains inconsistent and conversation-level.
Some records have no source property; later records use ad hoc source properties.
Asha's properties include duplicate conversation references under different keys.
Stable message-level source tracking remains unfinished.

The test makes paid model and embedding calls. Provider-wide cost is not captured here.

## Evidence

- [Full transcript](transcript.md)
- [Readable graph snapshot](graph.md)
- [Per-turn graph snapshots and timings](report.json)
- [Final graph nodes](nodes.csv)
- [Final graph relationships](relationships.csv)
- Raw SSE responses: `turn-01.sse` through `turn-20.sse`.

Inspect the retained world in Memgraph:

```cypher
MATCH p=(a {world_id: '01a1873c-dd93-7dba-a440-e45a4e9563ff'})-[r]->(b)
RETURN p;
```
