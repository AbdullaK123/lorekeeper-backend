# Auth-token repository tests — 2026-11-07

**Eight real Postgres tests pass in 0.80 seconds.**

- Valid tokens can be consumed once; stored hashes match the returned raw secret.
- Expired tokens are rejected.
- Reissue replaces the previous link and refreshes its expiry.
- Wrong-user, wrong-purpose, and wrong-hash attempts fail without consuming the valid link.
- Two concurrent consumers produce exactly one success.
- Password-reset lifetime is 30 minutes; email-verification lifetime is 24 hours.

[Raw results](results.txt). Tests: [auth_tokens.rs](../../cases/auth_tokens.rs).

The recreated lorekeeper_adversarial database receives all seven migrations.
Test fixtures require that isolated database name and delete their QA users after success.
Cleanup verification finds zero token test users and zero auth-token rows.
Development app_db data is untouched. No emails or model calls occur.
SQLx preparation for all targets succeeds with warnings in 22.26 seconds and refreshes query metadata.
Formatting and whitespace checks pass. No full suite or browser rerun occurs.
These tests cover repository behavior, not account-update transactions, reset endpoints, sessions, or email delivery.
