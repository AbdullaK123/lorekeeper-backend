# Second 20-turn smoke — 2026-10-29

## Result

Repeated the same Brinewake scenario in a fresh account, world, and conversation.
All 20 chat turns and all 20 background researcher runs completed without reported errors.
The stored conversation has 40 messages with 40 distinct IDs.

The graph contains **23 entities, 2 unresolved conflicts, and 38 relationships**.
Two relationships link conflicts; 36 connect lore entities.
The first run's world and evidence remain separate.

- World ID: `01a18769-1944-7007-bfcc-dcd431b35356`.
- Conversation ID: `01a18769-1953-7785-bf1a-1d436775cbaf`.
- Database: local `lorekeeper_adversarial`.
- Chat-plus-research time: 624.5 seconds.
- Median turn time: 32.65 seconds, including research.

## Source tracking checks

Checked every entity and lore relationship after every turn:

- No missing source lists.
- Every source ID matches an actual user message in this conversation.
- No assistant message IDs or foreign message IDs appear as sources.
- No duplicate IDs within source lists.
- No retained record loses its earlier source IDs across updates.
- Asha's source list accumulates turns 3, 5, and 11.
- The Nightjar keeps sources from turns 3, 7, and 19 after its route changes.
- Turns 8, 12, 15, 16, and 20 leave graph records and source lists unchanged.

Conflict nodes and system-created conflict links use the separate conflict storage path.
They do not yet carry message-level sources and are excluded from these entity/relationship source checks.

## Lore behavior

- Asha's birthplace update preserves her guild membership, key, and earlier source references.
- Bren receives the contextual Captain clarification; Asha remains a guild member.
- Iven's eye injury, storm year, nonmagical navigation, and negotiation restriction remain saved.
- Nessa, her relationship to Bren, and Button remain saved.
- The sealed-room meeting and negotiation restriction remain two unresolved conflicts.
- The hypothetical export ban adds no canon facts or sources.
- Old Nightjar salt-route edges disappear. Its new letter route to Kest is saved.
- The retired seasonal route remains historical detail in the ship's description.
- The final answer separates canon from unresolved contradictions.

## Limits

This is one sequential regression run, not a concurrency or recovery test.
Each research run finishes before the next user turn starts.
Recall answers have the full chat history; fresh-conversation retrieval is not tested.
Source lists identify supporting messages for whole records, not individual claims within a record.
The researcher still adds some redundant conversation-source metadata inside arbitrary properties.
The extraction granularity varies: this run creates separate mine and strait records.
This does not by itself indicate a duplicated subject.

## Evidence

- [Conversation with message IDs](transcript.md)
- [Graph with source turn references](graph.md)
- [Source audit](source-audit.json)
- [Per-turn snapshots and timings](report.json)
- [Raw node export](nodes.csv)
- [Raw relationship export](relationships.csv)

The synthetic world remains in Memgraph. Inspect it with:

```cypher
MATCH p=(a {world_id: '01a18769-1944-7007-bfcc-dcd431b35356'})-[r]->(b)
RETURN p;
```
