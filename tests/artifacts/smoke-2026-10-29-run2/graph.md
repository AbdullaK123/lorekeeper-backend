# Final graph with source messages

World: `01a18769-1944-7007-bfcc-dcd431b35356`.

23 entities, 2 unresolved conflicts, 38 total relationships.

## Entities

### Brinewake

ID: `382a433a-7cae-4f52-8791-9b8053ab079f`.
Source user turns: 1.

A stormy island chain.

### Vesper

ID: `ccb13663-df18-4e63-b89c-006464a6cfb2`.
Source user turns: 1.

Brinewake's main port city, built around a tidal lighthouse.

### Vesper lighthouse

ID: `18f09cc9-0d1f-4860-a786-04aae26429af`.
Source user turns: 2.

The tidal lighthouse around which Vesper is built. Its lenses run on blue salt and it is maintained by the Lantern Guild.

### Lantern Guild

ID: `a4140448-57b9-47e2-9fad-b44a4a957bdd`.
Source user turns: 2, 7, 10.

Maintains Vesper's tidal lighthouse and buys blue salt from Orin's miners. It does not own the mines. During winter, when the strait freezes, the guild uses stored salt. The guild opposes the toll on blue salt shipments imposed in 319.

### Blue salt

ID: `a904844d-81e0-40dd-9f91-77fbdc115adb`.
Source user turns: 2.

Salt used to power the lenses of Vesper's lighthouse; mined on the island of Orin.

### Orin

ID: `31795bd3-db8b-4121-a619-b483ad06b17a`.
Source user turns: 2.

An island where blue salt is mined by its miners.

### Orin's miners

ID: `01f68fd0-b24a-490d-86d5-43195a8bd524`.
Source user turns: 2, 9.

Miners on Orin who mine blue salt; the Lantern Guild buys salt from them. They formed the Deep Bell Union.

### Orin's mines

ID: `623be29f-ae99-4f3a-b37e-8ee2e0ff0cd4`.
Source user turns: 2.

Mines on the island of Orin. The Lantern Guild does not own them; it buys blue salt from Orin's miners.

### Asha Pell

ID: `5affd0ca-9c42-4463-bed7-ad12e3c38d62`.
Source user turns: 3, 5, 11.

Asha Pell was born on Orin and moved to Vesper at age six, where she grew up. She captains the courier ship Nightjar and belongs to the Lantern Guild; her crew usually calls her Captain Ash. She has a small brass key sewn inside her left coat cuff; only her navigator, Iven Marr, knows she has it.

### Nightjar

ID: `36eac936-9ce0-4b27-9bf4-f77ae290dfa1`.
Source user turns: 3, 7, 19.

A courier ship captained by Asha Pell. Its former April-to-September blue-salt route from Orin to Vesper is retired; it now carries letters from Vesper to Kest, a southern island.

### Bren Vale

ID: `3866be6c-f8eb-4158-8abb-a95e3ddc64b2`.
Source user turns: 4, 10, 17, 18.

Bren Vale captains the fishing boat Petrel. People on the dock also call him Captain. He is unrelated to Asha Pell and is not a member of the Lantern Guild; he refuses to join because he distrusts Tal Ren's friends. He supports the toll on blue salt shipments imposed in 319 because its revenue will repair fishing piers. Nessa is his older sister and the Petrel's cook, not his employee on a formal wage.

### Petrel

ID: `3f555d2a-f6ee-476b-93e5-79e9ae67c110`.
Source user turns: 4, 17.

A fishing boat captained by Bren Vale, with his older sister Nessa as its cook. Nessa keeps a black cat called Button aboard.

### Small brass key

ID: `b21607b8-9ea1-4eb5-bfad-67f4d24554c8`.
Source user turns: 5.

A small brass key sewn inside Asha Pell's left coat cuff. It opens the old lens room beneath Vesper's lighthouse.

### Old lens room

ID: `6f9156cc-d754-4aa5-b17d-828367571776`.
Source user turns: 5, 13.

A room beneath Vesper's lighthouse, opened by Asha Pell's small brass key. It was permanently sealed by a rockfall in 318; nobody has reopened it, and it has no other entrance.

### Iven Marr

ID: `1b66c764-9ef7-438c-afa7-22338885a61e`.
Source user turns: 5, 6.

Asha Pell's navigator and the only person who knows she has a small brass key sewn inside her left coat cuff. He lost his right eye in the winter storm of 312. He navigates the Nightjar using tide charts, not magic.

### the strait

ID: `68ac0fc2-548d-43d7-92cd-bb7d89021cf9`.
Source user turns: 7.

Freezes during winter.

### Deep Bell Union

ID: `0d4726ec-3962-4ce9-951e-f485b1a0aba3`.
Source user turns: 9.

A union formed by Orin's miners.

### Sera Dain

ID: `3bc5f9a9-a1a4-4886-8445-bbff1e085eba`.
Source user turns: 9.

Leader of the Deep Bell Union. She refuses to sell salt to the military but honors civilian contracts with the Lantern Guild.

### Tal Ren

ID: `39a69478-e938-4ea3-93a4-f9face06cf92`.
Source user turns: 10.

Governor of Vesper; imposed a toll on blue salt shipments in 319.

### Blue salt shipment toll

ID: `08e37e67-40fc-4b69-812f-82226ba1d12b`.
Source user turns: 10.

A toll on blue salt shipments imposed in 319 by Vesper's governor, Tal Ren. Its revenue is intended to repair fishing piers. The Lantern Guild opposes it; Bren Vale supports it because the revenue repairs fishing piers.

### Nessa

ID: `d6b40b39-0aa2-4423-997e-0df62bff07c7`.
Source user turns: 17.

Nessa is Bren Vale's older sister and the cook aboard the fishing boat Petrel. She is not Bren's employee on a formal wage. She keeps a black cat called Button aboard.

### Button

ID: `5a5142e4-31b3-4604-8536-653ad05a027f`.
Source user turns: 17.

A black cat kept by Nessa aboard the fishing boat Petrel.

### Kest

ID: `10e6a815-29c5-4490-aea6-6b32a783b451`.
Source user turns: 19.

A southern island.

## Connections

| From | Relationship | To | Source user turns |
| --- | --- | --- | --- |
| Vesper | LOCATED_IN | Brinewake | 1 |
| Vesper lighthouse | LOCATED_IN | Vesper | 2 |
| Vesper lighthouse | MAINTAINED_BY | Lantern Guild | 2 |
| Vesper lighthouse | USES | Blue salt | 2 |
| Lantern Guild | BUYS_FROM | Orin's miners | 2 |
| Lantern Guild | USES | Blue salt | 7 |
| Lantern Guild | OPPOSES | Blue salt shipment toll | 10 |
| Blue salt | MINED_IN | Orin | 2 |
| Orin's miners | LOCATED_IN | Orin | 2 |
| Orin's miners | WORKS_IN | Orin's mines | 2 |
| Orin's miners | FORMED | Deep Bell Union | 9 |
| Orin's mines | LOCATED_IN | Orin | 2 |
| Asha Pell | CAPTAINS | Nightjar | 3 |
| Asha Pell | GREW_UP_IN | Vesper | 3, 11 |
| Asha Pell | MEMBER_OF | Lantern Guild | 3 |
| Asha Pell | POSSESSES | Small brass key | 5 |
| Asha Pell | BORN_IN | Orin | 11 |
| Nightjar | CARRIES_LETTERS_TO | Kest | 19 |
| Bren Vale | CAPTAINS | Petrel | 4 |
| Bren Vale | UNRELATED_TO | Asha Pell | 4 |
| Bren Vale | NOT_MEMBER_OF | Lantern Guild | 4, 18 |
| Bren Vale | SUPPORTS | Blue salt shipment toll | 10 |
| Small brass key | OPENS | Old lens room | 5 |
| Old lens room | LOCATED_BENEATH | Vesper lighthouse | 5 |
| Old lens room | HAS_CONFLICT | Conflict 887c0bb3-1c00-4d6c-9e4e-c6f20976ad9e |  |
| Iven Marr | NAVIGATOR_FOR | Asha Pell | 5, 6 |
| Iven Marr | KNOWS_ABOUT | Small brass key | 5 |
| Iven Marr | NAVIGATES | Nightjar | 6 |
| Iven Marr | HAS_CONFLICT | Conflict 90dfbc4b-92ed-4982-87a6-e5369cb88e56 |  |
| Sera Dain | LEADS | Deep Bell Union | 9 |
| Sera Dain | HONORS_CIVILIAN_CONTRACTS_WITH | Lantern Guild | 9 |
| Tal Ren | GOVERNOR_OF | Vesper | 10 |
| Tal Ren | IMPOSED | Blue salt shipment toll | 10 |
| Blue salt shipment toll | LEVIED_ON | Blue salt | 10 |
| Nessa | OLDER_SISTER_OF | Bren Vale | 17 |
| Nessa | COOKS_ABOARD | Petrel | 17 |
| Nessa | KEEPS | Button | 17 |
| Button | ABOARD | Petrel | 17 |

## Unresolved conflicts

- `887c0bb3-1c00-4d6c-9e4e-c6f20976ad9e`: accessibility / events after 318. Resolved: false.
  The writer supplied both claims as notes but explicitly has not decided how to reconcile them. A 320 meeting inside the room conflicts with its stated permanent sealing in 318, no reopening, and no other entrance.
- `90dfbc4b-92ed-4982-87a6-e5369cb88e56`: contract negotiation authority. Resolved: false.
  The writer supplied both claims as notes but explicitly has not decided how to reconcile them. Negotiating a salt contract conflicts with the established restriction that Asha does not let Iven negotiate contracts.
