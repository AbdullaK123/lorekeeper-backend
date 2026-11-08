# Recovery HTTP verification — 2026-11-08

Added an opt-in HTTP recovery workflow test using the full application router and real QA Postgres/Memgraph.
Requests use Tower oneshot through the router and middleware; email uses an actual loopback HTTP capture server.
The installed Resend SDK supports RESEND_BASE_URL, so no production injection or controller-state changes are needed.
The workflow passes in 5.43 seconds: logged-out known/unknown requests return the same response; invalid email sends nothing.
It checks email recipient and API_BASE_URL, missing query inputs, weak passwords, invalid secrets, successful reset without a session, old-password rejection, new-password login, and replay rejection.
An initial harness failure came from decoding &amp; but not Askama's numeric ampersand entity. The harness now handles both. Application code stays unchanged.
The successful run removes its user; two users from failed harness attempts are also removed from QA. Login session records follow normal expiry.
Formatting and git diff --check pass. No real emails, paid model calls, browser tests, rebuilds, or deployments occur.
The unrelated known retrieval-ranking failure is not rerun. This targeted test does not make the full suite green.
Confirmation pages, existing-session revocation, and broader recovery abuse controls remain pending. Product scope and epic acceptance stay unchanged.
