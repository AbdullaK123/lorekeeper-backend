# Conflict message sources — 10-turn smoke, 2026-10-29

## Result

All ten chat turns and ten background research runs complete without reported errors.
The fresh Ardent Reach world contains **13 entities, two unresolved conflicts, and 16 lore relationships**.
Two additional links attach conflicts to their subjects.

- World: `01a18777-7732-7461-8c9f-6545cd1c666e`.
- Conversation: `01a18777-7744-715a-a990-daf20fc93f3f`.
- Database: local `lorekeeper_adversarial`.
- Stored messages: 20, each with its own ID.
- Total chat-plus-research time: 269.6 seconds.
- Median turn: 26.2 seconds, including research.

## Verified behavior

| Check | Result |
| --- | --- |
| Aqueduct contradiction | Unresolved conflict references turn 5's exact user message |
| Archive access contradiction | Unresolved conflict references turn 10's exact user message |
| Trusted source assignment | Source IDs come from tool context, not model arguments |
| Source validity | Every entity, conflict, and lore relationship source resolves to a user message in this conversation |
| Source duplicates | None appear in the smoke snapshots |
| Birthplace correction | Mira changes from Sablecross to Greyhaven; her job, compass, and friendship remain |
| Relationship correction | Mira's current BORN_IN edge targets Greyhaven |
| Temporal qualifiers | FEEDS and MAINTAINS descriptions and conditions explicitly say before destruction in 204 |
| Contextual identity | Oren receives the soldiers' permit restriction; Lysa receives the daylight-only access rule |
| Hypothetical isolation | Turn 8 changes neither graph facts nor source lists |
| Recall isolation | Turn 9 changes neither graph facts nor source lists |
| Canon preservation | No repair, alternate aqueduct, or midnight exception enters established lore |

## Implementation and focused checks

Conflicts now expose `source_message_ids`, including through full entity reads.
The repository appends each reporting message once and preserves earlier IDs.
Legacy conflicts without that field read as an empty source list.
A focused real-Memgraph test proves accumulated sources, retry deduplication, and legacy reads.

## Limits

Source IDs on a conflict identify the messages reporting that conflict.
They do not separately identify the original source for each disputed claim.
The original established facts remain available through the subject's source references.
System-created HAS_CONFLICT links do not carry their own source lists.

This is a sequential smoke test. It does not test concurrent writes, durable recovery, or deletion races.
Recall also has full conversation history; it does not prove fresh-conversation retrieval.

## Evidence

- [Full transcript](transcript.md)
- [Graph and conflict source turns](graph.md)
- [Per-turn snapshots and source audits](report.json)
- [Nodes](nodes.csv)
- [Relationships](relationships.csv)

The synthetic world remains in Memgraph for inspection:

```cypher
MATCH p=(a {world_id: '01a18777-7732-7461-8c9f-6545cd1c666e'})-[r]->(b)
RETURN p;
```
