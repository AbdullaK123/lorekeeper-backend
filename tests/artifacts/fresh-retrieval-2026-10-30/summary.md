# Fresh-conversation retrieval — 2026-10-30

All five reviewed answers pass. Each question starts in a separate empty conversation.
The fixture contains only synthetic Ardent Reach lore. No original history enters these chats.
All five chat requests and background research runs finish without reported errors.
Before and after graph snapshots match.

| Check | Result |
| --- | --- |
| Minor facts and corrected birthplace | Greyhaven, adult move, compass in right boot, Oren knows |
| Relationship conditions | Council ownership, destroyed in 204, no current supply |
| Shared Warden title | Separate roles and correct access rules |
| Unresolved conflicts | Both saved conflicts, both versions, no invented resolution |
| Unknown facts | Population and reservoir reserves remain unspecified |

The conflict response calls `get_conflicts` with `{}`.
Its tool result contains the two saved unresolved conflicts and their message sources.
The answer preserves the aqueduct and archive contradictions.
This resolves the demonstrated failure from the previous fresh-conversation run.

These are manual answer checks, beyond the harness completion assertions.
This run does not test restart recovery or concurrent research writes.

- [Questions and answers](transcript.md)
- [Saved messages and graph snapshots](report.json)
- [Conflict tool call and result](unresolved_conflicts.sse)

The synthetic world and five new conversations remain available for inspection.
