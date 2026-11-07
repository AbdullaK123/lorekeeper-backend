# Auth bootstrap and transaction verification — 2026-11-07

**All 14 focused Postgres tests pass in 6.64 seconds.**

Bootstrap now constructs AuthTokenRepository and supplies it, the pool, user repository,
and configured Resend key to AuthService. No other production logic is changed by this task.

The original eight token tests now acquire a connection and pass it to consume_token.
Six new checks cover:

- Successful service reset commits a usable password hash and consumes its token.
- Invalid and expired secrets leave the password unchanged.
- An injected password UPDATE failure rolls back token deletion; retry with the same secret succeeds.
- Replaying a consumed secret through the service cannot change the password again.
- Explicit verification rollback restores both verified=false and the consumed verification token.

The verification rollback assertion confirms the returned user was verified inside the transaction.
The error assertion respects the application's sanitized database-error response; an earlier test-only expectation of raw error text is corrected.

[Raw results](results.txt). [Tests](../../cases/auth_tokens.rs).
All-target compilation passes with warnings. SQLx all-target preparation passes with warnings in 19.86 seconds.
Formatting and whitespace checks pass. The offline query cache includes the new account-update queries.
Cleanup verification finds no token QA users, token rows, injected triggers, or injected functions.
Development data is untouched. No emails, model calls, browser run, or full suite run occur.

These checks do not complete the password-reset HTTP flow, email delivery, session-revocation policy,
or an email-verification service/endpoint. Verification coverage here tests repository transaction rollback.
