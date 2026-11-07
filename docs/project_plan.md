# Lorekeeper — Project Plan

## Auth logging gaps — 2026-11-07

Added auth outcome events for rejected requests/tokens, transaction commits, rollbacks, and transaction failures.
Added email rendering, provider submission, failure, and acceptance events. Acceptance does not imply delivery.
Token logs distinguish stored tokens from consumption inside an uncommitted caller transaction.
Events omit credentials, token hashes, addresses, URLs, payloads, and raw error text.
Updated logging guidance. Both existing HTTP/error logging capture tests pass in 0.01 seconds.
The test target compiles with existing warnings; whitespace checks pass.
New auth outcome events are reviewed in source; their runtime branches are not exercised here.
No email/provider calls or database transaction tests occur.
Auth delivery priorities, remaining work, story scope, and epic acceptance stay unchanged.

## Auth tracing coverage — 2026-11-07

Added 19 missing tracing::instrument attributes across auth handlers, services, and repository writes.
All new spans skip arguments. Explicit context contains only user_id where available.
No raw errors or return values are added to these spans. Updated docs/logging.md.
All-target offline compilation passes with existing warnings in 19.47 seconds.
Whitespace checks pass. No tests, email sends, or provider calls occur.
Remaining auth delivery work and epic acceptance stay unchanged.

## Configured auth email base URL — 2026-11-07

Settings now requires API_BASE_URL. Bootstrap passes it into AuthService.
Password-reset and verification links use that value and preserve user_id and token.
The constructor removes trailing slashes before adding /auth paths.
Added the local development value to .env when absent and documented it in README.
Compose already loads .env; no container restart or rebuild occurs.
This replaces the earlier hardcoded-localhost URL limit. Browser confirmation pages remain pending.
All-target offline compilation passes with existing warnings in 35.74 seconds.
Whitespace checks pass. No tests or emails run for this configuration change.
Delivery priorities, story scope, and epic acceptance remain unchanged.

## Auth email endpoints — 2026-11-07

Added POST /auth/password_reset/request and /auth/email_verification/request for the signed-in user.
Added public POST /auth/password_reset and /auth/email_verification with validated JSON.
Confirmation requires user_id and token; password reset also requires new_password under the signup policy.
Added user_id to generated email links. README documents routes and request fields.
All-target offline compilation passes with existing warnings. Four request-validation tests pass.
All 14 isolated Postgres token and transaction tests pass in 8.63 seconds.
No emails are sent. No browser or live HTTP check occurs in this turn.
Limits: localhost links still need browser confirmation pages and a configured public URL.
Logged-out reset requests by email address and existing-session revocation remain pending.
Story scope, priorities, and epic acceptance remain unchanged; account recovery is not complete.

## Verification service review — 2026-11-07

Reviewed the verification sender, template binding, token consumption, and user update.
By inspection, verification consumes the correct token purpose and updates the user in one transaction.
The verification link points to localhost and an unregistered route. HTTP wiring remains pending.
The invalid-token message in verify_user still says reset token.
Next: wire the verification request and confirmation routes with the required user identity.
No application code changes, tests, or email sends occur in this review.
Delivery priorities, story scope, and epic acceptance remain unchanged.

## User verification email template — 2026-11-07

Added `templates/email_verification.html`, matching the password-reset email design.
The Askama input is `verification_url`. Includes a verify link, fallback URL, and unexpected-signup notice.
Isolated Askama rendering and escaping checks pass.
Local Chromium checks pass at 390px and 1440px: no horizontal overflow and both links preserve the URL.
Email-client rendering remains unchecked. No email is sent. No backend logic changes occur.
Template struct and verification sender now exist; see the verification service review below.
Story scope, delivery priorities, and epic acceptance remain unchanged.

Updated: 2026-11-03.

## Extensive description browser verification — 2026-11-03

All 28 Playwright checks pass in 192.75 seconds on a rebuilt local debug API.
A fresh QA world receives 20 paid chat turns and background research, plus two description rewrites.
Accepted age 41 now agrees with the description, which retains unrelated facts and removes age 34.
Both threads answer 41. Fifteen extended questions preserve the full entity and source IDs after research completes.
Repeated acceptance, resolved-conflict reads, exact history after reload, mobile inspection, and displayed errors pass.
No uncaught browser errors occur. The old stale-description finding is fixed in this tested scenario.
Evidence: [extensive browser summary](../tests/artifacts/playwright-extensive-description-2026-11-03/summary.md).
No full suite rerun or Docker rebuild occurs. The temporary API stops; QA evidence remains.
Next: verify container packaging. Concurrent edits, search indexing, and broader conflict targets remain unchecked.
All story requirements and epic acceptance states stay unchanged.


## Stale embedding fix — 2026-11-03

A free Memgraph test reproduces the bug: corrected descriptions retain the old stored search vector.
WorldService now embeds the corrected description before opening the acceptance transaction.
Entity::embedding_text shares the name/kinds/description format with researcher graph writes.
The repository saves the replacement vector, description, property, sources, and decision together.
Embedding failure returns before graph writes. Repeated acceptance keeps the first committed vector.
Verification: all four conflict regressions pass in 1.34 seconds. All-target offline compilation, formatting, and whitespace checks pass.
The rebuilt image includes this fix. All 16 dev/prod runtime checks pass, including clean exit 0.
Image: lorekeeper-backend:verified-20261103. No Compose rollout or hosted deployment occurs.
Evidence: [image and embedding verification](../tests/artifacts/docker-verification-2026-11-03/summary.md).
No paid browser rerun or full suite rerun occurs for this change.
Superseded task: historical vector repair is unnecessary after the user-requested database reset.
Use a one-off command scoped to an explicit world and entity list, with a preview before paid embedding calls.
Recompute vectors from current entity text. Preserve descriptions, properties, decisions, and source IDs.
Start with known affected QA entities; no production-wide corruption is established.
Concurrent-edit protection remains separate unfinished work.
This planning turn changes no application code and runs no tests. No repair or model calls occur yet.
Frontend work stays deferred. Story scope and epic acceptance stay unchanged.

## Clean development reset — 2026-11-03

User requests a full local reset instead of repairing historical QA vectors.
Removed this project's Compose containers, network, and both persistent database volumes.
Recreated Postgres and Memgraph; applied all six application migrations to app_db.
Rebuilt the API image through the cached Docker build and started the Compose API with ENV=dev.
Compose build fails because the Buildx plugin is missing; docker build followed by compose up --no-build succeeds.
Verified /health, /, and /api/health return 200. Both databases report healthy.
Postgres has zero users, worlds, and conversations; Memgraph has zero nodes.
Historical QA databases and stale vectors are deleted. Disk-based test evidence remains.
The isolated lorekeeper_adversarial database must be recreated before running its native tests again.
No paid calls or full suite run occur. Other projects remain untouched; scope and epic acceptance stay unchanged.
The historical-vector repair task is superseded by this reset. Future acceptance writes retain the embedding fix.

## Backend next-step recommendation — 2026-11-03

Recommended next feature: password recovery, covering US-042 and US-044 within Epic 9.
Auth routes currently provide signup, login, logout, and profile reads; no password recovery routes exist.
Start by designing request/response models for requesting a reset and submitting a new password.
Use expiring, single-use tokens stored as hashes. Keep account-existence details out of reset-request responses.
Resend is selected. Sender verification is confirmed by the user; template setup remains before email integration. Frontend work remains deferred.
This is a recommendation, not completed or started implementation. No application changes or tests run this turn.
Product scope and epic acceptance stay unchanged. The clean dev stack is the current baseline.

## Resend SDK introduction — 2026-11-04

User chooses Resend for the next email integration and reports having an API key.
Reviewed official Rust SDK and sending documentation. No secret file is read.
Teach client creation, email options, and async send before integrating password recovery.
The resend-rs dependency is not present at this inspection. No dependency or application code changes occur.
No emails are sent and no tests run. Sender-domain verification remains unconfirmed.
Account recovery, token storage, and email integration remain unfinished. Scope and epic acceptance stay unchanged.
Sources: https://resend.com/docs/send-with-rust and https://github.com/resend/resend-rust.

## HTML email template guidance — 2026-11-04

User owns a Cloudflare domain; Resend sender verification is not yet confirmed.
Resend is the selected provider. Recommended initial option: hosted, published Resend templates.
The backend supplies a template ID and variables such as RESET_URL; Resend renders the email.
Alternative: keep HTML templates in the repository, render them in Rust, and send with with_html.
Template storage choice remains a proposal. No dependency changes, DNS edits, API calls, emails, or tests occur.
Domain verification and password-recovery implementation remain unfinished. Epic acceptance stays unchanged.
Reference: https://resend.com/blog/introducing-templates.

## Askama email template proposal — 2026-11-04

User confirms their Cloudflare domain is verified in Resend.
Recommend Askama for repository-owned email templates, with Resend handling delivery.
Askama compiles templates from templates/ into Rust; render() returns text for Resend with_html.
HTML templates use automatic escaping. Template changes require a rebuild.
The Docker builder must copy templates/ before cargo build; runtime template files are not required.
Hosted Resend templates are an alternative, not the proposed initial integration.
No dependencies, code, DNS, or emails change this turn. No tests run. Epic acceptance stays unchanged.
Reference: https://askama.rs/en/stable/creating_templates.html.

## Askama macro diagnostic — 2026-11-04

Inspected rust-analyzer settings and the new auth email module without reading secrets.
The reported Askama macro shared-library file is absent from rust-analyzer's separate target directory.
Recommend restarting rust-analyzer to reload and rebuild macro artifacts.
A syntax-only rustfmt check also reports an unfinished send_password_reset_email function declaration.
Started a separate offline Cargo check, then stopped it during dependency compilation; no completed build result is claimed.
No application code changes, tests, or email sends occur. Template integration remains unfinished.
Product scope and epic acceptance stay unchanged.

## Password reset email review — 2026-11-04

Reviewed the new auth email module and locally installed Askama/Resend APIs.
The intended flow is template data, HTML rendering, email options, then async send.
Syntax check fails: missing semicolon after template construction and extra closing parenthesis in with_html.
Askama rendering returns its own Result; Resend with_html accepts &str. The current Resend-only return type does not cover render errors.
The root password_reset.html file is empty. Docker template copying remains unfinished.
No implementation edits, emails, or tests run. Only a syntax check occurs; no passing build is claimed.
Story scope and epic acceptance stay unchanged.

## Password reset HTML template — 2026-11-04

Created templates/password_reset.html using the existing reset_url field and Askama escaping.
The layout uses inline styles, presentation tables, readable text, a reset button, and a fallback URL.
Copy includes an unexpected-request notice and no unimplemented expiry promise.
Docker now copies templates/ into the builder before compiling Askama templates.
An isolated Askama 0.16.1 render check passes, including dynamic text escaping.
Local Chromium checks pass at widths 1440 and 390: both links retain the supplied URL, with no horizontal overflow.
Preview screenshots: tests/artifacts/email-template-2026-11-04/. Mobile preview is visually inspected.
No email is sent. Real email-client rendering, full application compilation, and Docker rebuild are not tested here.
The sender function's syntax and render-error handling remain unfinished. Scope and epic acceptance stay unchanged.

## EmailService review — 2026-11-04

EmailService now owns a reusable Resend client and maps render/send failures into ServiceError.
The constructor declares resend_api_key but passes undefined api_key to Resend::new; this prevents compilation.
The send response is unused; the function intentionally returns unit on successful submission.
Syntax/format checking passes for email.rs. No full compiler check, emails, or tests run this turn.
EmailService has no caller yet. Recovery routes and token handling remain unfinished.
Product scope and epic acceptance stay unchanged. No application code edits occur.

## AuthService email ownership review — 2026-11-04

AuthService now owns a private EmailService and constructs it from the supplied Resend key.
Follow-up review confirms bootstrap supplies settings.resend_api_key and the sender returns Ok(()).
Syntax/format checks pass for the auth and email modules. No full build result is claimed.
Next slice: define reset-token storage before connecting the sender to recovery requests.
The email service still has no recovery-method caller. Token persistence and recovery routes remain unfinished.
Read-only review only: no application edits, compiler run, tests, or emails occur.
Story scope and epic acceptance stay unchanged.

## Auth-token schema sketch — 2026-11-04

Proposed auth_tokens columns: token_hash TEXT primary key, user_id UUID foreign key to users with cascade deletion,
purpose TEXT restricted to password_reset/email_verification, created_at TIMESTAMPTZ, and expires_at TIMESTAMPTZ.
Use a unique random token in the email and persist its SHA-256 hash only; no extra row ID is needed.
Add a user_id index for account token invalidation. Require expires_at > created_at.
Consume by deleting the row. Password change and token consumption must share one transaction.
The migration is not written or applied in this schema-sketch turn. No code, tests, or email sends occur.
Expiry duration and sibling-token invalidation rules remain to be chosen. Epic acceptance stays unchanged.

## Auth-token migration review — 2026-11-04

Reviewed migrations/20261104131140_auth_tokens_table.sql without applying it.
CONSTRAINT UNIQUE (user_id, purpose) lacks a constraint name; use UNIQUE directly or provide a name.
Follow-up: expired_at no longer has a default, so callers must supply expiry explicitly.
Explained PostgreSQL future-time syntax with NOW() + INTERVAL '30 minutes' as an example, not a chosen policy.
Explained both unique-constraint forms: UNIQUE (user_id, purpose), or CONSTRAINT auth_tokens_user_purpose_unique UNIQUE (user_id, purpose).
No migration changes or SQL execution occur in this teaching turn. Migration validity remains unverified.
user_id and purpose are nullable; user deletion lacks ON DELETE CASCADE.
The added users.verified flag is nullable. One token per user/purpose requires an issuance replacement rule.
Next review focus: correct constraint syntax and choose explicit future expiry semantics.
No migration edits, database execution, tests, or emails occur. Epic acceptance stays unchanged.

## Updated auth-token migration review — 2026-11-04

The migration now uses valid UNIQUE syntax and a five-minute expiry default.
Follow-up review confirms CHECK parentheses and CREATE INDEX syntax are corrected.
Final source review confirms user_id/purpose are NOT NULL, token deletion cascades with user deletion,
and users.verified is NOT NULL DEFAULT FALSE. The reviewed migration is ready for database validation.
The unique user/purpose constraint permits one outstanding token per purpose; issuance must replace an existing token.
Next: apply/test the migration, then add token repository methods. No further schema changes are required by this review.
SQL is reviewed only; no migration execution or database validation occurs this turn.
user_id, purpose, and users.verified remain nullable; the user foreign key still lacks cascade deletion.
Five minutes applies to both purposes under the current default; different lifetimes are not implemented.
Read-only review: no SQL edits, migration execution, tests, or email sends occur. Epic acceptance stays unchanged.

## Auth-token repository formatter diagnostic — 2026-11-07

Reproduced rustfmt's syntax error in src/data/repositories/auth_token.rs.
The let result = sqlx::query_as!(...) statement ends without a semicolon at line 31.
This is a source parse error, not evidence of a rust-analyzer failure.
The query and method return remain unfinished; adding the semicolon only fixes parsing.
No application edits, full compiler check, database tests, or email sends occur.
Product scope and epic acceptance stay unchanged.

## Auth-token hashing guidance — 2026-11-07

Recommend SHA-256 for high-entropy random reset/verification tokens; keep Argon2 for passwords.
Use the sha2 crate and persist the full 32-byte digest as 64 lowercase hexadecimal characters in token_hash.
Recommend nanoid!(32) with its default secure generator/alphabet; do not replace it with a non-cryptographic generator.
Email the raw token only. On submission, hash it identically, check purpose and expiry, and consume it atomically with the account change.
The hashing helper is synchronous and does not need repository state.
Consulted OWASP forgot-password/session guidance and RustCrypto/nanoid documentation.
No dependencies, application code, tests, database rows, or emails change this turn. Epic acceptance stays unchanged.

## Auth-token upsert conflict rule — 2026-11-07

Use ON CONFLICT (user_id, purpose) DO UPDATE to replace the existing token hash and both timestamps.
EXCLUDED contains values from the attempted insert, including evaluated timestamp defaults.
Replacement invalidates the previous link for that user/purpose and starts a fresh expiry window.
Other purposes remain unchanged. Do not silently ignore token-hash primary-key conflicts.
The repository query remains unfinished. No application edits, SQL execution, tests, or emails occur this turn.
Product scope and epic acceptance stay unchanged.

## Auth-token expiry recommendation — 2026-11-07

Recommend password-reset links valid for 30 minutes and initial email-verification links valid for 24 hours.
These are proposed app policies, not universal security requirements. OWASP requires suitable expiry and single use without one fixed duration.
Set expiry explicitly by purpose when issuing; the shared five-minute schema default does not express those different lifetimes.
Validate expired_at > database NOW() during atomic consumption. Expired rows need not be deleted immediately to be rejected.
Reissuing replaces the hash and resets timestamps; viewing a link must not extend its lifetime.
No migration/code edits, SQL execution, tests, or emails occur. Epic acceptance stays unchanged.
Reference: https://cheatsheetseries.owasp.org/cheatsheets/Forgot_Password_Cheat_Sheet.html.

## Rust token-expiry calculation — 2026-11-07

The project already uses chrono 0.4.45 and SQLx chrono support.
Teach Utc::now() plus Duration::minutes(30) or Duration::hours(24), producing DateTime<Utc> for expired_at.
Select the duration from AuthTokenPurpose, capture now once, and bind the resulting timestamps into SQLx.
This teaching turn makes no application changes and runs no tests, SQL, or email sends.
Product scope and epic acceptance stay unchanged.

## SQLx offline cache diagnostic — 2026-11-07

The new query has no matching .sqlx metadata, while rust-analyzer explicitly sets SQLX_OFFLINE=true.
Explain online migration/schema validation and cargo sqlx prepare with SQLX_OFFLINE=false, followed by normal offline checks.
Current query must first replace DO SET with DO UPDATE SET and use the migration column expired_at consistently.
The purpose durations are reversed relative to the recommendation: email verification currently gets 30 minutes and password reset 24 hours.
The Rust method also remains incomplete, including imports/type conversions and its return value; preparation needs compilation to succeed.
No cache generation, migrations, source edits, compiler runs, or emails occur in this guidance turn.
Product scope and epic acceptance stay unchanged.

## Live SQLx development checks — 2026-11-07

Confirmed Compose Postgres is healthy and reachable on localhost:5433/app_db.
SQLx migration run succeeds; all seven migrations are already applied, including auth_tokens.
Verified token_hash, user_id, purpose, created_at, and expired_at exist in the live table.
Updated local rust-analyzer cargo.extraEnv to SQLX_OFFLINE=false and the local Compose DATABASE_URL.
Incremental compilation remains disabled. Docker's offline build setting is unchanged.
The online all-target compiler check reaches Postgres and rejects the new insert: column expires_at does not exist; the live schema uses expired_at.
This confirms live SQLx query validation works. The full build fails with additional unfinished Rust imports/types and return handling.
The SQL changes during the check now include DO UPDATE SET. No repository logic is edited by this task.
Existing .sqlx cache files are preserved; prepare is deferred until compilation succeeds.
Restart rust-analyzer if the editor still reports offline mode after the setting change.
No repository business logic changes, data reset, emails, or paid calls occur. Epic acceptance stays unchanged.

## Rust-analyzer recovery and tuning — 2026-11-07

Found the concrete check blocker in the language-server log: --jobs was supplied by both cargo.extraArgs and check.extraArgs.
Cargo rejected that duplicated flag before checking any code. Removed check.extraArgs duplication; inherited jobs now appears once.
Restarted only this workspace's analyzer after confirming its process cwd and 4.7 GB resident memory use.
Configured API-only on-save checks, one Cargo job, syntax-tree cache capacity 32, two analysis threads, and one requested macro process.
Excluded generated/build/artifact directories from analyzer scanning and file watching.
Kept live local SQLx, build scripts, Askama/SQLx macro expansion, formatting, and compiler diagnostics enabled.
Verified installed extension accepts all setting names and its configuration log loads the new values.
Askama and SQLx compiled macro libraries exist in target/rust-analyzer.
Ran the corrected Cargo command against that directory: 531 compiler artifacts, 42 build-script results, and valid JSON diagnostics.
The check completes with 24 real source errors, including missing auth-token imports and model/type issues. No passing application build is claimed.
Final sampled analyzer RSS remains about 4.4 GB; tuning is not a hard memory cap or proof of stable long-term memory use.
Full test-target checks now require a separate cargo check --all-targets run. No application code edits, tests, email, or data resets occur.
Product scope and epic acceptance stay unchanged.
Reference: https://rust-analyzer.github.io/book/configuration.

## Auth-token purpose SQLx binding — 2026-11-07

AuthTokenPurpose derives Serde traits only; those control serialization, not SQLx Postgres encoding.
The migration stores purpose as TEXT. Recommend binding an explicit &str representation rather than adding a database enum.
A synchronous as_str(&self) maps PasswordReset to password_reset and EmailVerification to email_verification.
Bind purpose.as_str() as the third query parameter. The duplicate sqlx_core note does not supply the missing enum encoding.
No source changes, compiler runs, tests, migrations, or email sends occur in this guidance turn.
Product scope and epic acceptance stay unchanged.

## SHA-256 trait import — 2026-11-07

The reported digest error requires bringing sha2::Digest into scope alongside Sha256.
Digest provides the digest associated function; importing Sha256 alone does not expose trait methods.
Guidance only: no source edits, compiler runs, tests, database operations, or email sends occur.
Product scope and epic acceptance stay unchanged.

## SHA-256 hexadecimal output correction — 2026-11-07

sha2 0.11.1 returns an array type without LowerHex, so the earlier format!("{:x}", digest) example fails.
Recommend the hex crate and hex::encode(Sha256::digest(token.as_bytes())) for lowercase hexadecimal storage.
Checked the installed hex API: encode accepts AsRef<[u8]> and returns String.
No dependency/source changes, compiler runs, tests, database operations, or emails occur this turn.
Product scope and epic acceptance stay unchanged.

## Direct token-hash expression — 2026-11-07

Clarified that hex::encode already returns the final String; remove any enclosing format!("{:x}", ...) call.
Use let token_hash = hex::encode(Sha256::digest(token.as_bytes())); with sha2::{Digest, Sha256} imported.
No application edits, compiler runs, tests, database operations, or email sends occur in this guidance turn.
Product scope and epic acceptance stay unchanged.

## SQLx enum mapping follow-up — 2026-11-07

AuthTokenRow now stores AuthTokenPurpose instead of String, so both query input and output need explicit SQLx enum mapping.
Recommend deriving sqlx::Type with type_name=text and rename_all=snake_case for the existing TEXT column.
The query macro input uses purpose as _; the RETURNING purpose column uses the AuthTokenPurpose type override.
Serde rename attributes alone do not supply SQL encoding/decoding. No database enum or dependency-version changes are required.
Also observed a missing semicolon on the Serde import. Guidance only; no source edits, builds, tests, SQL, or emails occur.
Product scope and epic acceptance stay unchanged.

## SQLx enum derive attribute diagnostic — 2026-11-07

The enum has the sqlx helper attribute but still lacks sqlx::Type in its derive list.
Explain that use sqlx does not enable helper attributes; derive sqlx::Type enables them.
The Serde import still lacks its semicolon. No application edits, compiler checks, tests, SQL, or emails occur.
Product scope and epic acceptance stay unchanged.

## SQLx enum result override — 2026-11-07

The enum now derives sqlx::Type, but the upsert still uses RETURNING *.
SQLx infers the TEXT purpose column as String; explicitly alias purpose as "purpose: AuthTokenPurpose" in RETURNING.
Use the existing SQLx decoder instead of adding an infallible From<String> conversion for a fallible enum parse.
The input purpose binding still needs the previously discussed purpose as _ override if macro type checking rejects it.
No application edits, builds, tests, SQL execution, or email sends occur. Epic acceptance stays unchanged.

## SQLx enum input type override — 2026-11-07

The query macro expects &str for the TEXT parameter even though AuthTokenPurpose now implements SQLx encoding.
Bind purpose as _ to use the custom enum encoder while bypassing the macro's inferred Rust parameter type.
This is SQLx query-macro override syntax, not conversion of the enum into a string.
No application edits, builds, tests, database operations, or email sends occur. Epic acceptance stays unchanged.

## SQL versus Rust enum-override placement — 2026-11-07

The user placed purpose as _ inside the INSERT column list; Postgres rejects that syntax.
Keep the SQL column name purpose unchanged. Put purpose as _ in the Rust query argument list after user_id.
The RETURNING purpose type alias remains valid SQLx macro syntax and should stay.
No application edits, compiler runs, SQL execution, tests, or email sends occur. Epic acceptance stays unchanged.

## Auth-token enum binding corrected — 2026-11-07

Corrected the one unchanged Rust query argument from purpose to purpose as _.
Live SQLx cargo check confirms the enum-binding error is gone; the auth-token query produces no reported error.
The full application check still fails on three User model errors: query results include verified but User has no such field.
No other application code changes, tests, data writes, migrations, or email sends occur.
Product scope and epic acceptance stay unchanged.

## SQLx cache refreshed and macro environment diagnosed — 2026-11-07

Generated current .sqlx metadata with online cargo sqlx prepare -- --all-targets --jobs 1; preparation succeeds with warnings.
Verified SQLX_OFFLINE=true cargo check --offline --all-targets --jobs 1 passes with warnings in 0.54 seconds.
The new auth-token query cache entry is present, alongside updated user-query metadata.
Confirmed the recurring cause: analyzer macro workers still inherit SQLX_OFFLINE=true while workspace Cargo settings request false.
Added explicit rust-analyzer.server.extraEnv with online mode and the local database URL.
Restarted only this workspace's analyzer; VS Code automatic restart still reuses old initialization/process environment.
Fresh logs still show the obsolete --jobs 2 --jobs 2 and --all-targets invocation, while the workspace settings specify one inherited --jobs 1 and no all-targets check.
This confirms the running VS Code client still uses its old initialization settings.
Developer: Reload Window is required; restarting only the analyzer process has already proved insufficient.
No further configuration changes or compiler runs occur in this follow-up; the previous offline all-target check passes.
The prepared cache resolves current offline compilation. No application code edits, data writes, tests, or email sends occur.
Product scope and epic acceptance stay unchanged.

## Token-consumption syntax diagnostic — 2026-11-07

New formatter logs show SQLX_OFFLINE=false and the correct local database URL, confirming those settings now load.
The earlier missing fn is already fixed in the current consume_token declaration.
Current source still has an unfinished let result macro call without its semicolon, and sqx instead of sqlx.
Explain these as source errors. Short loop-turn warnings alone do not establish an analyzer failure.
Read-only review: no application edits, compiler runs, tests, SQL, or emails occur. Epic acceptance stays unchanged.

## Auth-token repository review — 2026-11-07

Reviewed completed upsert_token and consume_token methods. DELETE ... RETURNING consumes a matching row atomically.
Follow-up source review confirms consume_token now correctly requires expired_at > NOW().
Follow-up confirms upsert_token now returns (AuthTokenRow, String) and Ok((result, token)); the raw token is retained for email delivery.
The tuple order is internally consistent. Only the hash is persisted.
Follow-up confirms the intended lifetimes are now implemented: EmailVerification gets 24 hours and PasswordReset gets 30 minutes.
Next verification: real database tests for single use, expired-token rejection, and reissue invalidating the old token.
Source review does not yet establish these behaviors through tests.
Account updates and token consumption still need a shared transaction before service integration.
No source changes, compiler runs, or database tests occur in this follow-up.
Additional pending findings: upsert_token discards the generated raw token, durations remain reversed relative to the recommendation,
and consumption uses the pool rather than participating in the account-update transaction.
No service currently calls these methods. Prioritize expiry validation before integration.
Read-only review: no application edits, compiler runs, tests, SQL execution, or emails occur. Epic acceptance stays unchanged.

## Auth-token repository database verification — 2026-11-07

Added eight real Postgres tests in tests/cases/auth_tokens.rs and registered them in adversarial_live.
All eight pass in 0.80 seconds: single use, expired-token rejection, reissue invalidation, wrong-user/purpose/hash rejection,
concurrent consumption with exactly one winner, and the 30-minute/24-hour lifetime policy.
The raw token matches its stored SHA-256 verifier and is not stored in plaintext.
Created the isolated lorekeeper_adversarial database and applied all seven migrations after the earlier reset.
QA cleanup leaves zero token users and zero auth-token rows. Development data remains untouched.
All-target SQLx preparation succeeds with warnings in 22.26 seconds; cache includes the current consume query.
Formatting and whitespace checks pass. No application code changes, emails, model calls, full suite, or browser tests occur.
Evidence: [token test summary](../tests/artifacts/auth-tokens-2026-11-07/summary.md).
Remaining: service/endpoints, session handling, and shared transactions for token consumption plus account updates.
Product scope and epic acceptance stay unchanged.

## Shared account/token transaction design — 2026-11-07

User identifies that token consumption and password/email-verification changes must commit together.
Follow-up confirms consume_token now accepts &mut sqlx::PgConnection and passes it directly to fetch_optional.
The pool fallback is removed, so the caller can supply the active transaction's connection.
Next implementation slice: UserRepository account-update methods must accept that same connection.
Recommend passing &mut PgConnection explicitly to participating repository methods and removing the pool fallback.
AuthService begins a transaction, consumes a valid token on its connection, applies the account update, then commits.
No matching token must prevent the account update. Any failure rolls back token deletion and account changes together.
Perform password hashing before opening the database transaction.
Prior eight repository tests predate this signature change; adapt them and add rollback/account-update coverage after implementation.
No code edits, compiler runs, tests, database writes, or emails occur in this design turn. Epic acceptance stays unchanged.

## Verify-user SQLx macro argument — 2026-11-07

UserRepository now has update_password and verify_user methods accepting the caller's PgConnection.
Follow-up source review confirms verify_user now supplies User as its query_as! output type.
Both account-update methods pass the supplied PgConnection directly into fetch_optional.
Next: equip AuthService with AuthTokenRepository and access to PgPool so it can own the account/token transaction.
Prior token tests need adaptation to the new connection parameter; no updated test pass is claimed.
update_password already shows the required query_as!(User, SQL, arguments) structure.
Read-only syntax diagnosis: no source edits, compiler runs, tests, SQL execution, or emails occur.
Shared-transaction service wiring remains unfinished. Epic acceptance stays unchanged.

## Auth-service recovery draft review — 2026-11-07

AuthService now owns PgPool, UserRepository, AuthTokenRepository, and EmailService as intended.
The reset-email draft creates a reset token and passes the raw token through the email link.
Primary next issue: reset_password accepts only user ID and new password; it needs the submitted raw token before proof can be checked.
Password hashing already occurs before opening a transaction. Consumption, account update, rollback, and commit are not implemented yet.
Bootstrap still uses the old AuthService constructor and needs the pool and token repository arguments.
The reset link is hard-coded to a localhost API URL; final client URL/configuration and endpoint behavior remain undecided.
Read-only review: no code edits, builds, tests, SQL execution, or emails occur. Epic acceptance stays unchanged.

## SQLx transaction refresher — 2026-11-07

reset_password now accepts the raw token; its transaction body remains unfinished.
Explain pool.begin(), passing &mut *tx to connection-based repository methods, commit(), and explicit rollback()/rollback on drop.
Only commit after both valid token consumption and a successful account update. Convert missing rows into errors.
Keep hashing before begin. SQLx begin/commit errors need conversion through the application's error boundary.
No implementation edits, builds, tests, database writes, or email sends occur. Epic acceptance stays unchanged.

## Reset-password transaction draft review — 2026-11-07

The draft shares one transaction connection between token consumption and the password update; expensive hashing occurs first.
Follow-up confirms token consumption is guarded with ok_or_else and errors are preserved through e.into() after rollback.
Invalid/expired tokens now stop before the password update in the reviewed source.
The Ok(None) branch now returns a controlled missing-user error, fixing its return type.
Follow-up confirms the Ok(None) rollback now propagates errors with ?;.
All reviewed reset-password branches now commit or roll back as intended; runtime behavior still needs tests.
Next blocking wiring change: bootstrap must construct AuthTokenRepository and pass pool, user repository, token repository, and key into AuthService::new.
No source edits or updated build/test results are claimed by this review.
Read-only review only: no application edits, compilation, tests, SQL execution, or emails occur. Epic acceptance stays unchanged.

## Auth bootstrap and reset transaction tests — 2026-11-07

Bootstrap now creates AuthTokenRepository and passes pool, user repository, token repository, and Resend key into AuthService.
Adapted the eight token tests to pass explicit connections. Added six service/repository transaction regressions.
All 14 tests pass in 6.64 seconds, including valid reset, invalid/expired rejection, replay rejection,
injected password-write failure with reusable token after rollback, and verification rollback restoring both records.
A test-only raw-error-text expectation is corrected to match the existing sanitized error boundary.
All-target compilation and SQLx preparation pass with warnings; metadata refresh takes 19.86 seconds.
Formatting and whitespace checks pass. No token test users/rows or fault-injection functions/triggers remain in QA.
Evidence: [transaction test summary](../tests/artifacts/auth-reset-transactions-2026-11-07/summary.md).
No reset HTTP endpoints, verification service, session policy, emails, model calls, browser tests, or full suite are completed by this task.
No production business logic changes beyond bootstrap wiring. Product scope and epic acceptance stay unchanged.

## Completion rule

The project is done when all 15 [epics](user_stories.md) pass their acceptance checks.
All 69 stories remain required. Delivery stages set the order, not a smaller product scope.

The [product vision](product_soul.md) defines the audience: writers who need reliable memory for large fictional worlds.
Canon means facts the writer establishes or confirms.

Backend routes alone do not complete a user story.
The client must expose the required behavior, including errors, source links, and graph views.
This repository contains backend work. The plan also names required client work.

No epic currently has recorded acceptance for its complete user workflow.
Working foundations remain valuable and should stay in place during changes.

## Planning record — 2026-11-03

Project instructions now require PM updates before every final response.
This turn changes documentation only. Product scope and delivery priorities stay unchanged.
The latest browser evidence supports one canon scenario, not complete epic acceptance.

## Current foundations

| Area | Implemented | Remaining limits |
| --- | --- | --- |
| Accounts | Signup, login, logout, sessions, current user | Duplicate signup now returns 409; complete privacy acceptance remains |
| Worlds | Create, list, read, update, delete; owner checks | Request text validation is enforced; deletion can race with background lore writes |
| Conversations | Create, list, resume, delete; streamed replies | Long histories, interrupted streams, and concurrent turns need defined behavior |
| Message storage | User and assistant messages save together before background research starts | Research status and retry APIs are built; chat/job handoff and fork/mapping persistence remain separate commits |
| Lore storage | Entities, relationships, conflicts, search, graph repository methods | Message sources persist on entities, relationships, and conflicts; broader identity acceptance and write coordination remain |
| Retrieval | Text and vector search, neighbor properties, relationship conditions | Conflict lookup passes five fresh-conversation checks; broader coverage remains |
| Graph APIs | Scoped graph, entity, neighbor, relationship, and conflict reads; supported conflict decisions | Product graph views and broader conflict targets remain |
| Sources and recovery | Source-message reads, research status, retry submission and current-attempt tracking | Client flows, crash commit gaps, and mapping deletion cleanup remain |
| Container packaging | Multi-stage playground/API build, non-root runtime, dev-only static files, tested graceful stop | Hosting, migrations during release, backups, and deployed acceptance remain |
| Logging | Structured request/service/repository/workflow logs and privacy capture tests | Production monitoring and alerting remain |
| Cleanup | Transactional deletion jobs, scheduled graph cleanup, retry after failure | Shutdown and competing background writes need further checks |
| Researcher | [Prompt](../prompts/researcher.md), scoped search/read/write/conflict tools, background runtime wiring | DBOS model/tool checkpoints implemented; interrupted writes and concurrent recovery still need acceptance |

The runtime uses a background tool-using researcher after successful chat storage.
Chat has read-only search access. The old structured extraction pipeline is removed.
Researcher jobs now run as DBOS workflows with model and tool checkpoints in Postgres.
Local scripted-provider tests verify DBOS engine restart replay. Atomic handoff and interrupted graph writes remain unfinished.

## Test baseline

Latest recorded verification on 2026-11-03:

- All-target offline compilation passes with warnings.
- 44 free native regressions pass in 12.94 seconds.
- Nine research retry/status HTTP tests pass in 9.475 seconds.
- The broader unit run has 11 passes and one duplicate-row retrieval-ranking failure.
- The accepted-canon browser regression passes all 10 steps in a fresh QA world.
- Both existing and fresh conversations answer the accepted age; research preserves saved entity data.
- Browser fix attempts use 13 paid chat turns and one embedding diagnostic.
- The full unit suite is not rerun. The Docker image still needs rebuilding.

These are separate scoped runs, not a combined full-suite pass count.
The duplicate-row assertion remains failing; no current query occurrence is demonstrated and no fix is scheduled.
Earlier paid smoke results remain in [test findings](../tests/REPORT.md) and their artifacts.
Source preservation, conflict targets, graph APIs, and restart replay have targeted evidence.
This evidence does not prove every identity case or exactly-once writes across databases.

## Epic completion checks

### Epic 1: Keep My Worlds Private and Separate

**Stories:** US-001, US-002, US-003, US-004.

**Built:** Account sessions, world management, owner checks, and queued graph deletion.
World deletion also removes its conversations in Postgres.

**Remaining:**

- Complete account and world screens, including useful validation errors.
- Verify isolation across every API and researcher tool.
- Prevent background research from recreating a deleted world's lore.
- Define how the client shows pending deletion and cleanup failures.

**Done when:** Two writers cannot access each other's data through any supported path.
Deleting a world removes its conversations and lore, even during active research or a restart.

### Epic 2: Build My World Through Conversation

**Stories:** US-005, US-006, US-007, US-008, US-009.

**Built:** Streamed chat, saved history, separate conversations, and read-only lore search during replies.
The prompts guide natural conversation and follow-up questions.

**Remaining:**

- Complete conversation navigation, note pasting, streaming, and resume behavior in the client.
- Define supported note size and history limits.
- Handle long conversations without losing relevant earlier context.
- Define concurrent turn ordering, interrupted replies, and safe retry behavior.
- Test follow-up questions against real worldbuilding cases.

**Done when:** A writer can paste notes, discuss them, leave, and resume without lost or duplicated turns.
Follow-up questions use the conversation and saved lore without inventing canon.

### Epic 3: Remember My Lore Without Manual Filing

**Stories:** US-010, US-011, US-012, US-013, US-014, US-015.

**Built:** Tool-using researcher, exact-ID batch writes, full reads, conflict reporting, message sources,
DBOS checkpoints, status polling, and retry APIs. Retry state tracks the current workflow attempt.

**Remaining:**

- Verify contextual identity through the tool-using researcher with behavioral evaluations.
- Preserve old facts, minor details, relationship conditions, and sources during updates.
- Resolve identity through context; ask for clarification when evidence remains unclear.
- Support distinct subjects with the same name without silently merging them.
- Connect the built status/retry APIs to client feedback.
- Close chat/job and fork/mapping commit gaps; define cleanup for mapping rows after message deletion.
- Protect updates against concurrent jobs and repeated processing.

**Done when:** Restarting or retrying research loses no accepted lore and creates no duplicate facts or relationships.
Ambiguous names never cause silent writes to the wrong subject.
The writer can see and retry failed saves.

### Epic 4: Find Answers I Can Trust

**Stories:** US-016, US-017, US-018, US-019, US-020.

**Built:** Natural-language retrieval, combined ranking, entity/neighbor properties, relationship conditions,
message source IDs, and an authorized source-message API.

**Remaining:**

- Complete claim-level answer citations and their client navigation.
- Link answer claims to readable source statements in the client.
- State unknown details clearly and distinguish unresolved claims from established facts.
- Test retrieval with indirect wording, minor details, and misleading nearby matches.

**Done when:** A writer can verify factual answers against their earlier statements.
Answers preserve conditions and direction, and identify missing evidence.

### Epic 5: Keep Control of Canon and Contradictions

**Stories:** US-021, US-022, US-023, US-024, US-025, US-026, US-027.

**Built:** Source-aware conflict records, scoped list/detail APIs, exact property-key conflict reporting,
and keep-existing/accept-new decisions. Accept-new updates the entity property and resolves the conflict atomically.

**Remaining:**

- Connect conflict read/decision APIs to client review.
- Extend supported conflict targets beyond existing entity properties.
- Define unclear-reference review and consistent changes across affected relationships.
- Keep hypothetical statements and unapproved assistant suggestions outside canon.
- Route unresolved identity questions back to the writer before attaching facts.

**Done when:** A writer sees the evidence, decides what remains canon, and verifies the resulting lore.
Unresolved issues remain visible across sessions without silently replacing established facts.

### Epic 6: See How My World Connects

**Stories:** US-028, US-029, US-030, US-031.

**Built:** Owner-scoped graph, entity, search, neighbor, relationship, and conflict HTTP reads.
Filtered pagination and populated graph fixtures have targeted tests.

**Remaining:**

- Build the visual graph and entity inspection views in the client.
- Support following connections across entity types.
- Show relationship direction, conditions, details, and source links.
- Keep large worlds usable through bounded loading and navigation.

**Done when:** A writer can explore their world visually and inspect a character's connections with supporting details.

### Epic 7: Reason About Consequences Within My World

**Stories:** US-032, US-033, US-034, US-035.

**Built:** Search and graph context provide a starting point.
Dedicated consequence reasoning has no recorded acceptance yet.

**Remaining:**

- Keep hypothetical reasoning separate from automatic canon writes.
- Retrieve the relevant chain of relationships and its conditions.
- Explain inferred consequences with the supporting facts and source links.
- Identify missing premises rather than inventing world rules.
- Require explicit writer confirmation before saving a proposed change.

**Done when:** A hypothetical trade disruption produces a traceable explanation grounded in saved dependencies.
The answer labels inferences and unknowns, and leaves canon unchanged without confirmation.

### Epic 8: Find Gaps and Choose What to Develop

**Stories:** US-036, US-037, US-038, US-039.

**Built:** Saved lore and search provide inputs. Dedicated gap review remains unbuilt.

**Remaining:**

- Compare related entities using their saved facts.
- Identify uneven detail without treating every difference as a mistake.
- Gather open questions and let the writer return to them later.
- Show the evidence behind each suggested gap.
- Present gaps as questions without generating canon answers.

**Done when:** A writer can compare factions, review missing details, and return to open questions later.
The system preserves the writer's choice about what needs development.

### Epic 9: Access and Recover My Account

**Stories:** US-040, US-041, US-042, US-043, US-044.

**Status:** Required scope; complete implementation and acceptance are not recorded.
Password authentication and sessions provide an existing foundation.

**Remaining:**

- Extend the existing account model with verified identities and recovery records.
- Build verification, OAuth, linking, and recovery flows with client feedback.

**Decisions before build:** Provider selection, account linking rules, token expiry, and recovery for accounts without passwords.

**Done when:** A writer can sign in, recover access, and link identities without exposing or duplicating their worlds.

### Epic 10: Manage My Subscription and Payments

**Stories:** US-045, US-046, US-047, US-048, US-049, US-050.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Add customer, subscription, billing-event, and access records.
- Build checkout, billing management, event processing, reconciliation, and plan screens.

**Decisions before build:** Prices, currencies, taxes, refunds, trial policy, plan changes, and payment-failure access rules.

**Done when:** Payment state reliably controls access, and writers can manage billing without losing their lore.

### Epic 11: Understand and Control Usage

**Stories:** US-051, US-052, US-053, US-054.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Build usage records, reservations, access checks, and worker budget enforcement.
- Add usage screens, renewal information, and clear limit responses.

**Decisions before build:** Allowance units, reset periods, retry charging, queued-work policy, and hard spending ceilings.

**Done when:** Concurrent chat and background jobs respect limits, and writers see accurate usage and renewal information.

### Epic 12: Receive Essential Service Emails

**Stories:** US-055, US-056, US-057.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Build templates, queued delivery, provider integration, and delivery status handling.
- Connect account and billing events to the required messages.

**Decisions before build:** Email provider, sender domain, required notices, and which system owns each billing notice.

**Done when:** Essential messages arrive reliably, and failures remain visible and recoverable.

### Epic 13: Operate and Recover the Service

**Stories:** US-058, US-059, US-060, US-061.

**Status:** Required scope; complete implementation and acceptance are not recorded.
Containers, health response, cleanup retries, and partial shutdown handling provide existing foundations.

**Remaining:**

- Add health checks, structured failure reporting, alerts, and release automation.
- Write incident, restore, migration, and rollback procedures.

**Decisions before build:** Hosting, alert ownership, acceptable downtime, acceptable data loss, and backup retention.

**Done when:** Operators detect failures and prove recovery within agreed downtime and data-loss targets.

### Epic 14: Protect and Manage My Data

**Stories:** US-062, US-063, US-064, US-065, US-066.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Build export and account-deletion jobs with visible progress and safe retries.
- Add access restrictions, rate limits, administrative audit records, and accurate policy pages.

**Decisions before build:** Export format, retention periods, deletion confirmation, provider data handling, and launch policy requirements.

**Done when:** Writers can export and delete data, and published privacy behavior matches deployed handling.

### Epic 15: Get Help and Resolve Service Problems

**Stories:** US-067, US-068, US-069.

**Status:** Required scope; complete implementation and acceptance are not recorded.

**Remaining:**

- Add support links, useful error references, and operational lookup tools.
- Add controlled retry actions and administrative audit records.

**Decisions before build:** Support channel, response expectations, administrative roles, and rules for authorizing private-content access.

**Done when:** Writers can report failures, and authorized operators can investigate and recover work without broad data access.

## Delivery order

### Stage 1: Define the research boundary

**Status:** Runtime boundary and tools are built; broader identity acceptance remains.

**Supports:** Epics 3, 4, and 5.

The researcher prompt and initial runtime tools are implemented.
The old extraction pipeline is removed. Message references and scoped read/write contracts are implemented.
Broader identity behavior and explicit unresolved-reference workflows still need acceptance.

1. Define stable entity references and source message references.
2. Define search and full-read results, including facts, relationships, conditions, conflicts, and sources.
3. Decide how distinct same-name subjects fit the current unique-name storage rule.
4. Define create, update, link, unlink, conflict, and duplicate-merge tool contracts.
5. Define structured results for confirmed saves, unresolved references, and failures.

Tools enforce world ownership and exact targets.
The researcher interprets meaning and ambiguity within those boundaries.
Do not give the conversational agent unrestricted lore writes.

**Exit check:** Example tool calls represent every researcher prompt example without guessing a target or losing source evidence.

### Stage 2: Build one researcher workflow

**Status:** Implemented and smoke-tested. The items below describe required acceptance coverage, not missing tool scaffolds.

**Supports:** Epics 3 and 5.

1. Add search and full-read tools first.
2. Add entity writes that preserve earlier facts and sources.
3. Add exact-target relationship writes and explicit retractions.
4. Add conflict reporting and safe duplicate repair.
5. Run the researcher against isolated test worlds.

Start with one saved statement, one entity, and a verified read-back.
Then add updates, alternate names, relationships, contradictions, and corrections as separate slices.

**Exit check:** Behavioral tests cover shared aliases, distinct same-name subjects, preserved facts, and repeated processing.
They also cover qualifiers, explicit corrections, speculation, and unresolved references.

### Stage 3: Make automatic research reliable

**Supports:** Epics 1, 2, and 3.

1. Save the research job with the conversation changes in one Postgres transaction.
2. Run jobs with bounded work, durable status, retries, and restart recovery.
3. Prevent competing jobs from overwriting newer accepted facts.
4. Coordinate deletion with research so stale work cannot restore deleted lore.
5. Connect the built research status/retry APIs to the client and close their recorded commit gaps.
6. Complete behavioral acceptance of the researcher now wired into the runtime.
7. Keep obsolete extraction and alias logic removed; add regression cases against the active tools.

A one-time world existence check does not protect a later graph write from deletion.
The design must cover that interval and retries after partial writes.

**Exit check:** Failure and restart tests prove recovery, visible failure, and complete deletion during active research.

### Stage 4: Complete trustworthy answers and canon review

**Supports:** Epics 2, 4, and 5.

1. Fix retrieval omissions and verify ranking behavior.
2. Carry sources from research through retrieval into answer links.
3. Add conflict listing, inspection, resolution, and unclear-reference review.
4. Complete client flows for source inspection, corrections, and unresolved issues.
5. Verify long conversations, note limits, and follow-up behavior.

**Exit check:** A writer can save lore, question it, inspect its sources, and resolve a contradiction.
The resulting answers reflect the writer's decision.

### Stage 5: Complete graph exploration

**Supports:** Epic 6.

1. Use the built scoped graph, entity, neighbor, relationship, and conflict APIs.
2. Build the visual graph and connection inspection views.
3. Add source navigation and bounded loading for large worlds.

**Exit check:** Complete every Epic 6 workflow using the client against a representative world.

### Stage 6: Complete reasoning and gap review

**Supports:** Epics 7 and 8.

1. Add hypothetical reasoning with evidence paths and missing-premise reporting.
2. Verify that hypothetical conversations do not enter canon automatically.
3. Add grounded comparisons, gap questions, and a persistent open-question view.
4. Test these behaviors with incomplete and conflicting lore.

**Exit check:** Consequences remain traceable, unknowns remain explicit, and all canon changes require writer intent.

### Stage 7: Complete SaaS launch workflows

**Supports:** Epics 9–15.

1. Decide sign-in providers, plans, allowances, email delivery, and hosting.
2. Build account verification and recovery with reliable service emails.
3. Build usage accounting before enabling paid model workloads.
4. Add Stripe checkout, billing events, reconciliation, and account access rules.
5. Complete export, account deletion, support, and restricted operational tools.
6. Prove monitoring, backup recovery, release rollback, and published policy accuracy.

Build essential operational controls alongside earlier stages, before dependent features accept real users.
Do not treat this stage as permission to delay recovery, privacy, or spending controls.
Use the [lifecycle design](design.md) for each epic’s release and maintenance checks.

**Exit check:** Account, payment, limits, email, support, and recovery workflows pass deployed acceptance.

### Stage 8: Close every epic

**Supports:** All epics.

1. Preserve verified duplicate-signup handling and typed request validation.
2. Complete account, world, conversation, and recovery screens.
3. Test ownership across every new route and tool.
4. Test interrupted streams, concurrent turns, worker recovery, and deletion races.
5. Test deployment shutdown signals, including SIGTERM, and active job behavior.
6. Verify fresh migrations, backup recovery, and useful failure reporting.
7. Record acceptance evidence for each of the 69 stories.

Work on foundation defects alongside earlier stages when they block a workflow.
Do not postpone privacy or data-loss defects until final acceptance.

**Exit check:** Every story passes its writer-facing workflow, including failure and recovery where relevant.
No required epic moves into a later version to declare the project complete.

## SaaS launch decisions

The added scope is required. Provider choices and commercial rules remain open.
No price, allowance, OAuth provider, or hosting vendor is selected by this document.
Stripe is the intended billing integration.

Choose these rules before building their dependent flows:

- Supported OAuth providers and account-linking behavior.
- Plan prices, included usage, renewal periods, cancellation, and failed-payment access.
- Email provider, sender identity, and ownership of each notice.
- Hosting, recovery targets, retention, privacy disclosures, and support ownership.

Record each choice in the matching design epic and verify current provider requirements during implementation.
The earlier code-size estimate excludes this added scope and needs revision.

## Immediate next pieces

Next-step review: verify the current changes in a rebuilt Docker image.
Check health, playground access under ENV=dev, asset blocking under ENV=prod, and clean shutdown.
The 28-step browser run verifies the local debug API, not the release image.
No implementation changes or tests run in this planning turn. Scope and epic acceptance stay unchanged.
User direction: focus on backend work. Defer the actual frontend, including graph visualization.
Keep all frontend stories in scope for later; do not schedule them as the next implementation slice.
Release image verification now passes all 16 checks after the description and embedding fixes.
Stale embeddings are reproduced and fixed for future acceptance writes. Existing stale vectors still need a repair plan.
Concurrent-edit protection and research/deletion coordination remain backend gaps.
This planning turn changes no application code and runs no tests. Epic acceptance stays unchanged.


Next verification: rerun the fresh-world Playwright acceptance flow with an explicit description assertion.
Require age 41 in saved properties, removal of the old age claim from the description, and agreement in both conversation answers.
After question research completes, require the accepted entity and source IDs to remain unchanged.
The existing browser harness does not fail on stale descriptions; strengthen that check before running.
This planning turn runs no tests or paid calls and changes no application code.
Scope and epic acceptance stay unchanged. Concurrent-edit protection remains separate unfinished work.


Implemented: accept_conflict now receives the generated description and saves it in the existing graph transaction.
Properties, description, message sources, and the conflict decision share that transaction.
The service passes model output text and maps model-client environment errors into ServiceError.
The free repository regression now checks description persistence and unchanged description on repeated acceptance.
Verification: focused free database regression passes (1 test, 1.38 seconds). All-target offline compilation passes with warnings. Formatting and whitespace checks pass.
It no longer exercises WorldService, which now makes a paid model call.
Stale-input protection and description-based search indexing remain unchecked.
No browser rerun or paid calls occur in this implementation turn. Scope and epic acceptance stay unchanged.


Transaction guidance: reuse GraphRepository::accept_conflict and its existing commit/rollback handling.
Add the rewritten description as an argument and include e.description in the existing entity SET query.
The property, description, sources, and conflict decision then commit in the same graph transaction.
The service now generates new_description, but the repository does not save it yet.
A stale-input check remains needed before claiming concurrent-edit protection.
No application code changes or tests run during this guidance turn. Scope and epic acceptance stay unchanged.


Request guidance: use a tool-free Rig AgentBuilder with the configured chat model.
Send the old description and accepted field/value as JSON data; request only revised description text.
Await prompt to obtain a String. Existing ServiceError conversions cover provider and prompt errors.
Prefer injecting the configured model from bootstrap instead of creating clients inside WorldService.
The current acceptance method contains an unfinished request expression. No application code changes or tests run here.
The atomic description write and stale-snapshot check remain unimplemented. Scope and acceptance stay unchanged.


Proposed description fix: WorldService reads the entity and conflict, then requests one focused description rewrite.
The model receives the existing description and accepted correction, with instructions to preserve unrelated facts.
Run that model call before opening the write transaction.
Extend accept_conflict to save the corrected property, description, sources, and decision together.
Reject a stale entity snapshot instead of overwriting changes made during the model call.
If rewriting fails, leave the conflict unresolved and the entity unchanged.
This is a proposal, not an implemented or verified change. No code or tests run this turn.
Scope and epic acceptance stay unchanged. Description-based search indexing also needs checking during implementation.

Next task: choose how accepting a conflict also updates the entity description.
The observed case saves age 41 while the description still says 34 years old.
Acceptance check: property, description, and answers agree after acceptance; question research preserves that result.
The implementation approach remains undecided. No code changes or tests run in this planning turn.
Product scope and epic acceptance stay unchanged.

Fresh-world recheck: all 10 browser steps pass again; both threads answer the accepted age 41.
Research preserves the accepted entity. The description still says 34 years old.
See [recheck evidence](../tests/artifacts/playwright-recheck-2026-11-03/summary.md).
No application changes or full suite rerun occur. Story scope and epic acceptance stay unchanged.

Next-step review: rebuild the API image, then verify health and environment-based playground serving.
The latest canon checks use the local debug API; they do not verify the current container image.
Then address the observed stale description after conflict acceptance.
This planning turn runs no application tests and changes no delivery acceptance states.

The 2026-11-03 canon rollback now has a passing [browser regression](../tests/artifacts/playwright-canon-decisions-2026-11-03/summary.md).
Current properties and resolved decisions override stale history; questions preserve saved lore in that scenario.
Description text can still lag behind an accepted property. Define summary consistency before declaring full canon acceptance.

The researcher pipeline, graph read APIs, supported conflict decisions, source lookup, logging,
and research status/retry endpoints are implemented. Do not restart the extraction redesign.

1. Connect product client flows for graph browsing, source inspection, conflict decisions, and research recovery.
2. Choose the next backend slice: long-history limits, broader conflict targets, or deletion/research coordination.
3. Close recorded recovery commit gaps and mapping-row cleanup before claiming full recovery acceptance.
4. Keep SaaS account, billing, usage, email, privacy, support, and operations scope visible.

The next implementation task needs a bounded acceptance check. No new task is started by this documentation update.
