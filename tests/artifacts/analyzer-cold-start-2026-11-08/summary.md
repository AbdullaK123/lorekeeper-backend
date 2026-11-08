# Analyzer cold-start investigation — 2026-11-08

## Delivered changes

- Added src/lib.rs. Binary and integration tests import shared application modules.
- Made two existing researcher entry points public so integration tests can cross the library boundary.
- Removed unused direct anyhow and clap dependencies; the full Cargo metadata package list shrinks from 823 to 810.
- Updated library-test commands and layout documentation.
- Restored native diagnostics, test-file analysis, and automatic Cargo reloads after inconclusive settings experiments.
- Temporarily enabled RA_PROFILE and RA_LOG for the next real VS Code startup.

## Checks

Final all-target test compilation passes with warnings in 54.95 seconds.
The initial separate terminal-cache dependency rebuild takes 6m 21s; this is not an editor-startup measurement.
Library tests: 13 pass, the existing duplicate-row ranking test fails.
Integration pure tests: 11 pass. Text-type tests: eight pass. Binary test target contains no tests.
All 59 free native regressions pass in 19.19 seconds against QA Postgres and local Memgraph.
No email sends, paid AI calls, browser run, Docker rebuild, or deployment occurs.
Whitespace checks pass. Existing test assertions and business logic stay unchanged.

## Remaining problem

Cold start is not confirmed fixed. Analyzer instances retain about 4 GB resident memory.
Headless profiling shows expensive VFS handling and repeated workspace refreshes, including discarded build data.
One source-loading handler takes about 28 seconds; these probes run alongside the editor and are not controlled latency comparisons.
The bounded headless probes do not obtain a working hover within their limits.
Excluding tests, disabling native diagnostics, and disabling automatic Cargo reloads do not establish an improvement and are reverted.
The shared-library change removes source duplication but has no proven editor-startup speedup yet.
Server restarts reuse the old startup environment: a process environment check finds no RA_PROFILE after a restart.
Next required user action: Developer: Reload Window once to activate profiling in the real editor process.
Inspect that startup trace before making further changes; remove temporary profiler variables afterward.
