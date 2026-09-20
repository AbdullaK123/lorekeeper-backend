# lorekeeper-backend

Logging & Tracing

This service uses tracing with tower-http's TraceLayer. Logs are emitted to stdout.

What was fixed
- Added a fmt layer to tracing-subscriber so events actually print.
- Default log level is info when RUST_LOG is not set.
- Tracing is initialized at program start in main.

How to run with logs

PowerShell (Windows):
- $env:RUST_LOG = "info,tower_http=info,axum=info"
- cargo run

Bash (Linux/macOS/WSL):
- RUST_LOG="info,tower_http=info,axum=info" cargo run

Examples
- You should see a startup line like:
  - INFO Started server on port 8000...
- When you hit /health, you should see HTTP request spans from TraceLayer.

Notes
- Customize verbosity via RUST_LOG, e.g., RUST_LOG=debug or module-specific filters like RUST_LOG="lorekeeper_backend=debug,tower_http=trace".