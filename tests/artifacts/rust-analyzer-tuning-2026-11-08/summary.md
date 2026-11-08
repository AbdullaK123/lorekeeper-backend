## Rust-analyzer responsiveness tuning — 2026-11-08

Measured the active analyzer at about 3.2 GB resident RAM and 1.3 GB process swap on a 12-thread host.
Language-server logs show garbage-collection pauses up to 2.28 seconds during file changes.
Updated workspace settings: four analysis workers, 64 cached syntax trees, and ignored instrument expansion from tracing-attributes.
Only the intentional attribute-expansion-disabled notice is suppressed; unlinked-file diagnostics remain enabled.
SQLx, Askama, other macros, check-on-save, and online database checks remain enabled.
Validated all rust-analyzer setting names against installed extension 0.3.3105 and parsed the workspace JSONC.
The extension log confirms receipt of four workers and the ignored instrument configuration.
Online SQLx API compilation passes with warnings in 1.06 seconds. Whitespace checks pass.
The existing process still has its old worker count and memory footprint; reload the VS Code window to fully apply startup settings.
Editing latency and memory improvements remain unmeasured. No application code, test run, email, or provider calls occur.
Delivery priorities, story scope, and epic acceptance remain unchanged.

