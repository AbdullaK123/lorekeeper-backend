# Email verification HTTP evidence — 2026-11-08

Extended the existing local-capture auth workflow with email verification checks.
No application code changes.

- Logged-out verification requests return 401 and send no email.
- Logged-in requests send the correct recipient and a link using API_BASE_URL and the matching user ID.
- Expired tokens return 422 and leave verified=false in Postgres.
- A replacement link can be issued after expiry.
- Invalid secrets and wrong-user attempts return 422.
- Valid confirmation works without a login cookie, returns verified=true, and persists the flag.
- Confirmation does not create a session.
- Replayed tokens return 422.
- Already-verified requests return 409 and send no further email.

The combined recovery/session/verification workflow passes in 9.84 seconds in dev.
The integration test target compiles with existing warnings. Formatting and git diff --check pass.
Tests use isolated QA Postgres/Memgraph and a loopback Resend capture server.
No real emails, paid model calls, browser tests, application changes, or deployment occur.
The full suite is not rerun; the previously known retrieval-ranking failure is unchanged.
Browser confirmation pages and product epic acceptance remain pending.
