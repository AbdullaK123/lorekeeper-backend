# Google OAuth tests — 2026-11-10

## Results

64 tests pass; two fail across the targeted runs:

- Auth repository: 20 pass, including Google user/session atomicity and email-collision rejection.
- New local token-provider service tests: nine pass.
- New callback HTTP tests: two pass, one fails.
- Library pure tests: 13 pass, one known retrieval-ranking failure.
- Integration pure tests: 12 pass. Text-type tests: eight pass.

## Confirmed callback bug

`google_missing_attempt_failure_clears_the_state_cookie` fails.
The real router returns 401 for matching browser/query state with no saved attempt, but returns no removal cookie.
The service error exits through `?`, bypassing oauth_state cleanup. Authentication is rejected correctly; the temporary cookie remains stale.
Cancellation clears the cookie correctly. A callback from a different browser is rejected.

## Passing provider checks

A local HTTP token server supplies synthetic HMAC-signed ID tokens; no Google discovery or Google token requests occur.
Tests verify saved state/nonce, SHA-256 PKCE challenge, exchange code/verifier, successful passwordless verified signup, usable local session, and attempt replay rejection.
Wrong nonce, signature, issuer, audience, expired identity tokens, missing email, unverified email, and expired login attempts are rejected.
The expired-attempt and replay paths do not contact the token server.
These tests exercise the crate's configured local verifier, not Google's live RSA keys or browser consent flow.

## Limits and cleanup

The existing duplicate-row retrieval-ranking failure remains unchanged.
No production business logic is changed; only tests and cfg(test) module declarations are added.
Test compilation succeeds after correcting the dummy workflow's error type. Formatting and git diff --check pass.
QA cleanup finds zero oauth-/google-prefixed test users. DBOS is not launched in callback fixtures; model services carry dummy credentials and are unused.
No real Google sign-ins, emails, paid AI calls, browser runs, or deployments occur.
Full provider/browser acceptance remains pending. Next: centralize callback cookie cleanup across all error responses and rerun the failing test.
