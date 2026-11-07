# Auth endpoint wiring checks — 2026-11-07

- All-target offline compilation passes with existing warnings.
- Four request-validation tests pass.
- All 14 isolated Postgres token and transaction tests pass in 8.63 seconds.
- No emails, model calls, browser checks, or live HTTP tests occur.

Request-email endpoints require a session. Token-confirmation endpoints use JSON user_id and token.
Reset confirmation also validates new_password against signup rules.
Local email links include user_id but still need browser confirmation pages and a configured public URL.
Logged-out reset requests by email address and session revocation remain pending.
