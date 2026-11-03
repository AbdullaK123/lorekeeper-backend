# Release image and embedding fix — 2026-11-03

**All 16 image checks pass. All four focused conflict regressions pass.**

Image: `lorekeeper-backend:verified-20261103`.
ID: `sha256:cb3a5dd169cec2e1556dfcd3cce11f014e3d951cca2928501767f8a546fae963`.
Size: 42.25 MiB.
The successful Rust release compile takes 12m 31s, plus image assembly.
The earlier build is stopped because it predates the embedding fix.

## Image verification

- Runtime user is `lorekeeper`, not root. Default environment is production.
- Image uses SIGINT for clean application shutdown.
- Runtime contains no `/app/.env`, Cargo, or Node.
- Dev and production containers both pass health checks and reject anonymous world reads with 401.
- Dev serves the playground, its JavaScript asset, and the `/api/health` alias.
- Production returns 404 for the root playground, asset URL, and `/api/health` alias.
- Both containers stop with exit code 0 and no OOM kill.
- Temporary verification containers are removed. Other services and database volumes remain untouched.

[Image check report](report.json) records each assertion and shutdown timing.
The image is tagged locally; no hosted deployment or Compose rollout occurs.

## Stale embedding reproduction and fix

A free Memgraph test first fails: accepting age 41 changes the description but retains the old vector.
The fix generates the replacement vector from the corrected description before the acceptance transaction.
Both researcher graph writes and conflict acceptance use `Entity::embedding_text`.
The transaction saves description, vector, property, sources, and decision together.
The regression confirms the exact replacement vector persists and repeated acceptance preserves it.
All four conflict regressions pass in 1.34 seconds. All-target offline compilation passes with warnings.
Formatting and patch whitespace checks pass.

[Before/after regression evidence](embedding-check.json) records the original failure and passing fix.
No paid browser or model-provider test is rerun for this change; database tests supply deterministic vectors.
The service's real provider call is compiled but not exercised by that deterministic test.
Existing stale vectors are not backfilled. Concurrent edits remain outside this fix.
