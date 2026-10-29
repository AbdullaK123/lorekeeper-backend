# Final Memgraph contents

World: `01a1873c-dd93-7dba-a440-e45a4e9563ff`.

21 entities, 2 conflicts, 36 relationships including conflict links.

## Entities

### Brinewake

ID: `469edfe4-830f-4b20-ad31-906f006a97e2`.

A stormy island chain.

### Vesper

ID: `8e5eb323-f581-4ce1-8cf4-8bceb6847598`.

Brinewake's main port city, built around a tidal lighthouse.

### Tidal lighthouse of Vesper

ID: `76dc2022-8211-4657-b177-03c04d118558`.

A tidal lighthouse around which Vesper is built. Its lenses run on blue salt.

### Lantern Guild

ID: `8ada2504-3940-4fa0-9142-7b15d6e2c8fa`.

The guild that maintains Vesper's tidal lighthouse and buys blue salt from Orin's miners; it does not own the mines. When the strait freezes in winter, the guild uses stored blue salt.

### Orin

ID: `9bac8802-f464-491c-b86e-9c90ec862af2`.

An island where blue salt is mined.

### Blue salt

ID: `8f0beecf-61cb-4419-99eb-e2030a4eafb7`.

A salt mined on the island of Orin that powers the lenses of Vesper's tidal lighthouse.

### Orin's miners

ID: `92474181-bc9f-4dc2-bf64-51e67969f1ea`.

Miners on Orin who mine blue salt; the Lantern Guild buys salt from them. They formed the Deep Bell Union.

### Asha Pell

ID: `731a80b4-1cd1-40e2-9fe7-618d14dd3932`.

Born on Orin, Asha Pell moved to Vesper at age six and grew up there. She captains the courier ship Nightjar and belongs to the Lantern Guild; her crew usually calls her Captain Ash. She keeps a small brass key sewn inside her left coat cuff; only her navigator, Iven Marr, knows she has it.

### Nightjar

ID: `b0cd3a42-d5b0-44dd-972d-da7d960aab40`.

A courier ship captained by Asha Pell. Its former route carrying blue salt from Orin to Vesper between April and September is retired. It now carries letters from Vesper to Kest, a southern island.

### Bren Vale

ID: `9a340887-524b-4387-9b69-d7e321c84b3a`.

Captain of the fishing boat Petrel. People on the dock call him Captain. He is unrelated to Asha Pell and is not a member of the Lantern Guild; he refuses to join because he distrusts Tal Ren's friends.

### Petrel

ID: `0d21b413-13c7-4726-8b6b-c8f12484a752`.

A fishing boat captained by Bren Vale.

### Iven Marr

ID: `05ff3f16-a6a5-4f4c-833b-8f226a7a345d`.

Asha Pell's navigator on the Nightjar; the only person who knows she has the small brass key sewn inside her left coat cuff. He lost his right eye in the winter storm of 312. He navigates using tide charts, not magic.

### Old lens room beneath Vesper's lighthouse

ID: `287f4345-79ad-4e32-9acc-6168a9aafe18`.

An old lens room beneath Vesper's tidal lighthouse, opened by Asha Pell's small brass key. A rockfall permanently sealed it in 318. Nobody has reopened it, and it has no other entrance.

### Asha's small brass key

ID: `085fe1cd-e389-4552-bce4-9349fb8600f4`.

A small brass key sewn inside Asha Pell's left coat cuff; it opens the old lens room beneath Vesper's lighthouse.

### Deep Bell Union

ID: `dd36af1e-c96f-4468-b053-6cb56bd240b3`.

A union formed by Orin's miners.

### Sera Dain

ID: `4e7f03b3-a66c-4627-9e62-3e2ba6943d5c`.

Leader of the Deep Bell Union. She refuses to sell salt to the military but continues to honor civilian contracts with the Lantern Guild.

### Tal Ren

ID: `967724cf-6088-446c-8d58-f4be41f76bab`.

Governor of Vesper.

### Toll on blue salt shipments

ID: `2521192a-96cc-4cd6-849e-31b02c7d959e`.

A toll on blue salt shipments imposed in 319 by Vesper's governor, Tal Ren. The Lantern Guild opposes it. Bren Vale supports it because the revenue repairs fishing piers.

### Nessa

ID: `f66f3aa9-5f75-441b-9921-530cd348de93`.

The Petrel's cook and Bren Vale's older sister. She is not his employee on a formal wage.

### Button

ID: `bff96558-5e46-4181-8b98-665de1d66329`.

A black cat kept by Nessa aboard the Petrel.

### Kest

ID: `bad6adf5-46ea-4e4e-9cb5-03059da8b067`.

A southern island.

## Connections

| From | Relationship | To |
| --- | --- | --- |
| Vesper | BUILT_AROUND | Tidal lighthouse of Vesper |
| Vesper | LOCATED_IN | Brinewake |
| Tidal lighthouse of Vesper | USES | Blue salt |
| Lantern Guild | MAINTAINS | Tidal lighthouse of Vesper |
| Lantern Guild | BUYS_FROM | Orin's miners |
| Lantern Guild | USES_STORED | Blue salt |
| Lantern Guild | OPPOSES | Toll on blue salt shipments |
| Blue salt | MINED_ON | Orin |
| Orin's miners | LOCATED_IN | Orin |
| Orin's miners | FORMED | Deep Bell Union |
| Asha Pell | CAPTAINS | Nightjar |
| Asha Pell | GREW_UP_IN | Vesper |
| Asha Pell | MEMBER_OF | Lantern Guild |
| Asha Pell | CONCEALS | Asha's small brass key |
| Asha Pell | TRUSTS_WITH | Iven Marr |
| Asha Pell | DOES_NOT_ALLOW_TO_NEGOTIATE | Iven Marr |
| Asha Pell | BORN_IN | Orin |
| Nightjar | CARRIES_LETTERS_TO | Kest |
| Bren Vale | CAPTAINS | Petrel |
| Bren Vale | SUPPORTS | Toll on blue salt shipments |
| Bren Vale | REFUSES_TO_JOIN | Lantern Guild |
| Iven Marr | NAVIGATES_FOR | Asha Pell |
| Iven Marr | KNOWS_ABOUT | Asha's small brass key |
| Iven Marr | NAVIGATES | Nightjar |
| Iven Marr | HAS_CONFLICT | Conflict 1070c47c-f099-430c-8dfe-f13f6854dd0e |
| Old lens room beneath Vesper's lighthouse | LOCATED_BENEATH | Tidal lighthouse of Vesper |
| Old lens room beneath Vesper's lighthouse | HAS_CONFLICT | Conflict ce6d5fcc-b943-4005-96b3-6fcd1157c676 |
| Asha's small brass key | OPENS | Old lens room beneath Vesper's lighthouse |
| Sera Dain | LEADS | Deep Bell Union |
| Sera Dain | HONORS_CIVILIAN_CONTRACTS_WITH | Lantern Guild |
| Tal Ren | GOVERNOR_OF | Vesper |
| Tal Ren | IMPOSED | Toll on blue salt shipments |
| Nessa | COOKS_ABOARD | Petrel |
| Nessa | OLDER_SISTER_OF | Bren Vale |
| Nessa | KEEPS | Button |
| Button | ABOARD | Petrel |

## Unresolved conflicts

- `ce6d5fcc-b943-4005-96b3-6fcd1157c676`: access/status in 320. Resolved: false.
  The writer supplied both as notes and explicitly has not decided how to reconcile them. Keep the established sealing claim intact pending resolution. Source conversation: 01a1873c-dda9-7bf5-a8c5-094ae62ee4c8.
- `1070c47c-f099-430c-8dfe-f13f6854dd0e`: contract-negotiation restriction. Resolved: false.
  Potential scope tension: the note says Iven negotiates a salt contract, while Asha's established restriction concerns her allowing him to negotiate contracts; whether this is a contract within that restriction is not specified. The writer has not decided how to reconcile the notes. Source conversation: 01a1873c-dda9-7bf5-a8c5-094ae62ee4c8.
