# Development build tuning — 2026-11-08

Enabled incremental development compilation, replaced full debug data with line tables, and set two Cargo jobs.
IDE and terminal caches stay separate. Release settings do not change.
Removed forced incremental disabling from normal development/test instructions.

Both API checks use online SQLx validation against the local development database.
The Rust dependency resolver runs offline. The target directory is target/rust-analyzer.

- Cache preparation after the profile change: 231.29 seconds wall time, including dependency rebuilding and lock waits.
- Repeated unchanged check: 5.18 seconds wall time, including a package-cache lock wait.
- Both checks pass with warnings.
- Workspace JSONC and inherited job arguments validate. Whitespace checks pass.

These numbers do not establish a before/after speedup, changed-source rebuild speed, or editor startup latency.
The new profile can still require a first dependency rebuild in the separate terminal build cache.
Line tables retain source-line backtraces but omit full debugger local/type details.
No application logic changes, tests, email sends, paid AI calls, or deployment occur.
