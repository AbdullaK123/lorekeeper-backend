# Ardent Reach graph with message sources

World: `01a18777-7732-7461-8c9f-6545cd1c666e`.

13 entities, 2 unresolved conflicts, 18 relationships including conflict links.

## Entities

### Ardent Reach

ID: `47246434-b9cd-4afa-ad76-9003866fd11f`.
Source user turns: 1.

A desert region.

### Sablecross

ID: `1016d1ca-4f7d-4d74-bdc6-6522d6bb1947`.
Source user turns: 1.

A city in Ardent Reach.

### Sablecross reservoir

ID: `2a3ca8de-f578-4c53-a2cf-78f147162aee`.
Source user turns: 1, 4.

A reservoir in Sablecross, historically fed by the Glass Aqueduct before its destruction in 204.

### Glass Aqueduct

ID: `0a05f0fd-2132-40c5-b10c-63a7f2b9d187`.
Source user turns: 1, 2, 4.

The Glass Aqueduct once ran from the Basalt Hills to feed the Sablecross reservoir. The Sablecross council owns it. Mira Venn maintained it before its permanent destruction by a landslide in 204. It has not been rebuilt, and there is no alternate aqueduct or bypass.

### Basalt Hills

ID: `2ec99b4b-9d81-4b3e-8da3-e77d88559be1`.
Source user turns: 1.

Hills from which the Glass Aqueduct comes.

### Mira Venn

ID: `7d2fec61-8487-4192-8bf3-db75accaaa8f`.
Source user turns: 2, 6.

An engineer who maintained the Glass Aqueduct before its destruction in 204. She was born in Greyhaven, a small town on the coast, and moved to Sablecross as an adult. She keeps a cracked jade compass inside her right boot; only her friend Oren Dask knows about it.

### Sablecross council

ID: `ccb30b2e-6e33-42d7-a6d2-a754da506f10`.
Source user turns: 2.

The owner of the Glass Aqueduct.

### Oren Dask

ID: `4f6cb7f9-103d-41a5-93d2-3ed615a419fa`.
Source user turns: 2, 3, 7.

Oren Dask guards the Sablecross reservoir and refuses to let soldiers draw water without a council permit. He is also called the Warden. He is Mira Venn's friend and the only person who knows she keeps a cracked jade compass inside her right boot.

### Cracked jade compass

ID: `7e23d4d7-7bfd-4ce7-9db3-91d1ebe3ee71`.
Source user turns: 2.

A cracked jade compass that Mira Venn keeps inside her right boot. Only her friend Oren Dask knows about it.

### Lysa Renn

ID: `29ecd357-690c-4292-b52b-ade3cecc7021`.
Source user turns: 3, 7.

Lysa Renn keeps the Sun Archive and allows civilian scholars to enter during daylight only. She is also called Warden and is unrelated to Oren Dask.

### Sun Archive

ID: `e7e97d97-f1a0-4a74-98e5-6f4cd1c78282`.
Source user turns: 3, 7.

The Sun Archive is kept by Lysa Renn, contains the original aqueduct plans, and admits civilian scholars during daylight only.

### Original aqueduct plans

ID: `d5476f0a-57e4-43dc-b873-003fd4024e30`.
Source user turns: 3.

The original aqueduct plans, kept in the Sun Archive.

### Greyhaven

ID: `70ce38c9-4472-43bc-8fd0-7733401ad249`.
Source user turns: 6.

A small town on the coast.

## Conflicts

### condition and water flow after 204 destruction

ID: `621c3c46-89c9-4c20-888c-dd09668fdef1`.
Reporting user turns: 5.
Resolved: false.

These notes conflict: the established record says the aqueduct was permanently destroyed in 204 and was not rebuilt, while the new note describes it operating in 206. The writer explicitly says the notes are unreconciled and requests a conflict flag without inventing a repair or alternate channel. Source conversation: 01a18777-7744-715a-a990-daf20fc93f3f.

### civilian scholar access hours

ID: `1779d9cf-5d1b-4f71-8d66-2a10d083325a`.
Reporting user turns: 10.
Resolved: false.

The writer identifies this as an unreconciled note and has not decided whether it is an exception or a mistake. Preserve the daylight-only rule as established canon; do not add an exception. Source conversation: 01a18777-7744-715a-a990-daf20fc93f3f.

## Connections

| From | Relationship | To | Source user turns |
| --- | --- | --- | --- |
| Sablecross | LOCATED_IN | Ardent Reach | 1 |
| Sablecross reservoir | LOCATED_IN | Sablecross | 1 |
| Glass Aqueduct | FEEDS | Sablecross reservoir | 1, 4 |
| Glass Aqueduct | RUNS_FROM | Basalt Hills | 1 |
| Glass Aqueduct | HAS_CONFLICT | Conflict 621c3c46-89c9-4c20-888c-dd09668fdef1 |  |
| Mira Venn | MAINTAINS | Glass Aqueduct | 2, 4 |
| Mira Venn | MOVED_TO | Sablecross | 6 |
| Mira Venn | FRIEND_OF | Oren Dask | 2 |
| Mira Venn | CARRIES | Cracked jade compass | 2 |
| Mira Venn | BORN_IN | Greyhaven | 6 |
| Sablecross council | OWNS | Glass Aqueduct | 2 |
| Oren Dask | KNOWS_ABOUT | Cracked jade compass | 2 |
| Oren Dask | GUARDS | Sablecross reservoir | 3 |
| Oren Dask | UNRELATED_TO | Lysa Renn | 3 |
| Lysa Renn | KEEPS | Sun Archive | 3 |
| Sun Archive | CONTAINS | Original aqueduct plans | 3 |
| Sun Archive | HAS_CONFLICT | Conflict 1779d9cf-5d1b-4f71-8d66-2a10d083325a |  |
| Original aqueduct plans | PLANS_FOR | Glass Aqueduct | 3 |
