# User repository transaction ownership

Implemented the planned ownership refactor in repositories/user/mod.rs and its private auth_tokens child.
UserRepository constructs the child and exposes issue_auth_token, reset_password_with_token, and verify_email_with_token.
The parent consumes the token and changes the user on one connection, then commits or rolls back.
AuthService no longer holds a pool or token repository. Hashing and email delivery stay outside transactions.
Bootstrap supplies only UserRepository and email settings to AuthService. Existing SQL and token lifetimes stay unchanged; no migration is needed.
All 14 existing auth database tests move beside the private child. Two added tests cover verification failure/retry/replay and concurrent password updates.
All 16 pass against the isolated QA database in 9.91 seconds. All-target compilation passes with existing warnings.
Changed Rust files pass formatting checks; git diff --check passes. No emails, model calls, browser tests, Docker rebuild, or deployment occur.
Evidence: tests/artifacts/user-repository-ownership-2026-11-08/summary.md.
Product scope and epic acceptance stay unchanged. Logged-out reset requests, confirmation pages, and session policy remain pending.
The separate analyzer cold-start issue remains open; this refactor makes no startup performance claim.


Checks:

- `SQLX_OFFLINE=true cargo test --offline --all-targets --no-run --jobs 2` passes.
- `SQLX_OFFLINE=true DATABASE_URL=postgresql://dev:password@localhost:5433/lorekeeper_adversarial cargo test --offline --jobs 2 --lib data::repositories::user::tests:: -- --ignored --test-threads=1`: 16 pass.
- Selected-file rustfmt checks and `git diff --check` pass.

This is a targeted regression run. The unrelated known retrieval ranking failure is not rerun.
