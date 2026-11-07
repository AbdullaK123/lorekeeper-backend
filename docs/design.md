# Lorekeeper — Design and Development Lifecycle

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


The [user stories](user_stories.md) define the required outcomes.
The [product vision](product_soul.md) defines the audience and purpose.
The [project plan](project_plan.md) tracks current work and delivery order.
This document describes each epic through six lifecycle stages.

The status notes below distinguish built backend behavior from intended product behavior.
Build lists remain lifecycle checklists; they do not mean every listed item is unfinished.
Open decisions require answers before the affected build work starts.

Each epic follows this cycle:

1. **Define:** State the writer's need, scope, and acceptance conditions.
2. **Design:** Describe data, system boundaries, user flow, and failure behavior.
3. **Build:** Deliver small pieces using existing foundations where possible.
4. **Test:** Verify behavior, failure recovery, privacy, and writer acceptance.
5. **Deploy:** Release compatible changes and verify the deployed workflow.
6. **Maintain:** Monitor behavior, fix regressions, and repeat the cycle when needs change.

The project reaches delivery completion when all epics pass acceptance in the deployed product.
Maintenance continues after delivery completion.

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

## Latest design evidence — 2026-11-03

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

Current lore appears as labeled system data. Retrieval includes resolved writer decisions.
The researcher receives `user_message` and treats questions as requests for information, not new facts.
A fresh-world browser test passes all 10 steps, including accepted-canon answers in two conversations.
Research preserves saved entity data after those questions. All 44 free native regressions pass.
Descriptions can still contain old values. Summary updates remain an open design decision.
The Docker image needs rebuilding. Deployed epic acceptance remains unfinished.
Next packaging check: image health, dev playground serving, and production asset blocking.
Then address stale descriptions after conflict acceptance. No design or scope change occurs in this planning turn.

This turn updates documentation only. The lifecycle stages and product scope stay unchanged.
PM updates now occur before every final response, as required by [project instructions](../AGENTS.md).

## Epic 1: Keep My Worlds Private and Separate

**Implementation status — 2026-11-02:** Backend account/world CRUD, ownership checks, typed validation, duplicate-signup handling, and cleanup jobs exist. Client acceptance and deletion/research coordination remain.

### 1. Define

**Stories:** US-001, US-002, US-003, US-004.

Writers need private accounts and separate worlds for unpublished lore.
They can name, describe, and delete a world.
Deletion must remove conversations and graph data, including data created by background work.

**Acceptance:** No supported path exposes another writer's data.
A deleted world stays deleted after retries and restarts.

### 2. Design

- Authenticate requests through account sessions.
- Check world ownership in every API and tool before accessing its data.
- Derive tool scope from trusted request or job context, not model-supplied ownership fields.
- Keep world deletion and cleanup job creation in one Postgres transaction.
- Retry graph cleanup safely after partial failure.
- Coordinate research writes with deletion so stale jobs cannot restore removed lore.
- Show account errors, world validation errors, and pending deletion in the client.

**Open decisions:** Deletion status visibility and coordination between research and deletion. Typed world-name validation is implemented.

### 3. Build

1. Reuse account, session, world, and cleanup foundations.
2. Fix duplicate-signup errors and agreed world validation rules.
3. Add missing ownership checks as new tools and routes appear.
4. Complete deletion coordination and account/world client flows.

### 4. Test

- Try foreign world IDs through every route and tool.
- Test expired sessions, logout, duplicate signup, and invalid world input.
- Delete a world during active research and during cleanup failure.
- Restart workers and verify complete deletion without affecting other worlds.

### 5. Deploy

- Apply compatible schema changes before dependent application changes.
- Verify session configuration and access controls in the deployed environment.
- Exercise signup, login, world updates, and deletion using isolated test accounts.
- Confirm cleanup runs and reports failures without exposing private content.

### 6. Maintain

- Monitor account failures, denied access, cleanup failures, and old cleanup jobs.
- Review ownership checks whenever a route or tool changes.
- Verify backup access controls and recovery procedures.
- Investigate any recreated data after deletion as a correctness defect.

## Epic 2: Build My World Through Conversation

**Implementation status — 2026-11-02:** Streamed chat, saved UUID messages, conversation reads, and scoped source-message lookup exist. Client navigation, long-history behavior, and concurrent/interrupted turn rules remain.

### 1. Define

**Stories:** US-005, US-006, US-007, US-008, US-009.

Writers describe lore naturally, paste notes, use separate conversations, and resume earlier discussions.
The conversation asks relevant questions without inventing canon.

**Acceptance:** Saved turns survive leaving and returning.
Interruptions and retries do not silently lose or duplicate accepted turns.

### 2. Design

- Keep the conversation service responsible for chat flow and message storage.
- Keep lore tools read-only during conversational replies.
- Define stable message references for sources and retries.
- Define stream events for partial text, saved completion, and failure.
- Show reply progress separately from background lore-save progress.
- Set note-size and history limits with clear client feedback.
- Define context selection for conversations larger than the model's input limit.

**Open decisions:** Concurrent turn ordering, interrupted reply storage, and supported note/history limits. Research retry identity uses a current-workflow mapping.

### 3. Build

1. Reuse conversation routes, streamed replies, and atomic turn storage.
2. Add stable message references and the agreed stream contract.
3. Complete chat navigation, note pasting, resume, and retry states.
4. Add long-history handling and improve follow-up behavior through evaluated examples.

### 4. Test

- Paste notes, exchange turns, leave, and resume.
- Interrupt streams before and after message storage.
- Retry the same request and send competing turns.
- Test input limits, long histories, and relevant follow-up questions.

### 5. Deploy

- Release stream changes with a compatible client.
- Verify streaming through the deployed proxy and its timeout settings.
- Check stored history and retry behavior using an isolated conversation.
- Verify that reply completion does not falsely report completed lore saving.

### 6. Maintain

- Monitor reply failures, response delay, disconnects, and message-storage failures.
- Review context quality as conversations grow.
- Keep examples of poor follow-up questions for regression checks.
- Update client handling whenever the stream contract changes.

## Epic 3: Remember My Lore Without Manual Filing

**Implementation status — 2026-11-02:** The old extraction pipeline is removed. The researcher uses scoped tools and DBOS checkpoints. Research status/retry APIs and current-attempt tracking are built and tested. Separate commits and deletion coordination remain limits; see [retry details](research_retry.md).

### 1. Define

**Stories:** US-010, US-011, US-012, US-013, US-014, US-015.

The system saves stated lore without manual filing.
Updates preserve earlier facts, minor details, and subject identity.
Failures remain visible and retryable.

**Acceptance:** Repeated processing and restarts neither lose accepted facts nor create duplicate subjects.
Unclear references never silently attach facts to the wrong subject.

### 2. Design

- Use the [researcher prompt](../prompts/researcher.md) to guide search, reading, interpretation, and updates.
- Keep Memgraph as the graph store during the replacement.
- Use exact saved entity references for writes and relationship endpoints.
- Preserve source references, prior facts, conditions, and unresolved conflicts during updates.
- Persist research jobs with saved turns in one Postgres transaction.
- Bound job work and preserve status across retries and restarts.
- Prevent competing jobs from overwriting newer accepted facts.
- Surface pending work, failures, and unresolved references to the writer.

**Open decisions:** Broader same-name identity acceptance, write coordination, and recovery after partial graph writes. Current tools use exact IDs and message source references.

### 3. Build

1. Define exact references and search/full-read tool results.
2. Build read tools, then entity writes, relationship writes, and conflict tools.
3. Verify one saved statement through graph read-back before widening behavior.
4. Add durable jobs, safe retries, status APIs, and client feedback.
5. Replace old extraction only after the researcher passes behavioral checks.

### 4. Test

- Test shared aliases, canonical-name collisions, and distinct subjects with identical names.
- Preserve minor facts and relationship conditions through repeated updates.
- Fail between graph writes, restart, and retry the job.
- Run competing jobs and delete the world during research.
- Verify that failed work stays visible and retries do not duplicate facts.

### 5. Deploy

- Introduce job and source storage before switching processing behavior.
- Evaluate the researcher in isolated worlds before enabling production writes.
- Keep one production writer pipeline active during the switch.
- Define how pending jobs continue or pause during rollback.
- Verify saved facts and job status after deployment.

### 6. Maintain

- Monitor job age, failures, retries, processing delay, and model cost.
- Track wrong-subject writes, lost facts, and duplicate records as correctness defects.
- Version prompts and tool contracts together.
- Recheck representative cases after model, prompt, or tool changes.

## Epic 4: Find Answers I Can Trust

**Implementation status — 2026-11-02:** Retrieval includes entity/neighbor properties and relationship conditions. Source IDs persist on lore and resolve through an authorized message endpoint. Claim-level answer citations and product navigation remain.

### 1. Define

**Stories:** US-016, US-017, US-018, US-019, US-020.

Writers ask ordinary-language questions and receive answers grounded in saved lore.
Answers preserve conditions and link claims to earlier statements.
Missing information stays explicit.

**Acceptance:** The writer can inspect the evidence for an answer.
The system does not present missing or disputed information as established fact.

### 2. Design

- Combine text search and vector search, which finds related meaning.
- Expand relevant results through saved graph connections.
- Preserve entity properties, neighbor properties, relationship direction, and conditions in answer context.
- Carry stable source references through storage, retrieval, and client links.
- Distinguish established facts, disputed claims, and unknown details.
- Show when relevant background research remains pending.

**Open decisions:** Claim-level citation format, product retrieval limits, and deleted-source presentation. Source lookup returns 404 after conversation deletion.

### 3. Build

1. Fix missing properties and relationship descriptions in retrieval output.
2. Verify ranking behavior when search results repeat.
3. Add source references to stored lore and retrieved results.
4. Add answer source links and source inspection in the client.
5. Evaluate unknown and disputed-answer behavior.

### 4. Test

- Ask indirectly worded questions about minor details.
- Test misleading search matches and repeated search rows.
- Verify each answer link reaches the supporting statement.
- Test time limits, conditional relationships, unknown facts, and disputed claims.
- Confirm retrieval never crosses world boundaries.

### 5. Deploy

- Verify search indexes and source storage before enabling citation output.
- Define how existing lore without sources appears to writers.
- Run a deployed question-and-source check against an isolated world.
- Keep answer formats compatible with the deployed client.

### 6. Maintain

- Monitor retrieval failures, answer delay, and broken source links.
- Collect unsupported-answer cases without logging private lore by default.
- Recheck answer quality after search, embedding, prompt, or model changes.
- Repair missing sources without inventing historical evidence.

## Epic 5: Keep Control of Canon and Contradictions

**Implementation status — 2026-11-02:** Conflict list/detail and decision APIs exist. Accept-new atomically updates an existing entity property and resolves the conflict. Broader field/relationship targets and client review remain.

### 1. Define

**Stories:** US-021, US-022, US-023, US-024, US-025, US-026, US-027.

Canon means facts the writer establishes or confirms.
Writers inspect contradictions, clarify references, and make deliberate corrections.
Suggestions and hypothetical statements remain outside canon until confirmed.

**Acceptance:** Both disputed claims remain available until the writer decides.
The decision updates affected lore without erasing unrelated facts.

### 2. Design

- Store disputed claims, their sources, affected subjects, and resolution state.
- Check relevant connected facts as well as one entity's fields.
- Keep unresolved claims separate from accepted facts.
- Provide conflict review, clarification, and explicit correction flows.
- Preserve the writer's decision and its source for later review.
- Reject stale resolution requests that would overwrite a newer decision.

**Open decisions:** Broader conflict targets, clarification state, and consistent updates across affected relationships. Existing property targets support keep-existing and accept-new.

### 3. Build

1. Reuse conflict storage and existing contradiction examples.
2. Add source-aware conflict tools and unresolved-reference results.
3. Add authorized listing, inspection, and resolution APIs.
4. Build client review and clarification flows.
5. Connect explicit corrections to safe lore updates.

### 4. Test

- Test direct contradictions and contradictions across connected facts.
- Distinguish a correction from an unresolved disagreement.
- Reject unapproved assistant suggestions as canon.
- Keep hypothetical statements outside canon.
- Resolve competing decisions safely and preserve unrelated facts.

### 5. Deploy

- Preserve existing unresolved conflicts during schema changes.
- Enable review only when sources and resolution writes work together.
- Verify an entire conflict review in the deployed client.
- Confirm rollback preserves recorded writer decisions.

### 6. Maintain

- Monitor failed resolutions and old unresolved issues.
- Review false conflict reports and missed contradictions.
- Keep regression cases for corrections, speculation, and ambiguous references.
- Preserve decision history when prompts or storage change.

## Epic 6: See How My World Connects

**Implementation status — 2026-11-02:** Scoped graph, entity, neighbor, relationship, and text-search APIs are built. Populated pagination and isolation tests pass. The product visualization and source-navigation client remain.

### 1. Define

**Stories:** US-028, US-029, US-030, US-031.

Writers explore a visual graph, inspect subjects, and follow their connections.
Connection details explain direction and conditions.

**Acceptance:** A writer can trace connections across characters, places, factions, and events.
Large worlds remain usable without loading the entire graph at once.

### 2. Design

- Add authorized graph browsing, search, detail, and neighborhood APIs.
- Bound result sizes and support loading more connections.
- Use stable entity and relationship IDs across API responses and client selection.
- Show connection direction, conditions, details, and source links.
- Provide clear empty, loading, failure, and partial-result states.
- Provide an accessible list view alongside the visual graph.

**Open decisions:** Product layout, navigation limits, and supported world size. Backend response and pagination contracts are implemented.

### 3. Build

1. Reuse graph repository reads behind scoped services and routes.
2. Build graph navigation and entity inspection views.
3. Add connection details and source navigation.
4. Add bounded loading, search, and accessible navigation.

### 4. Test

- Verify owner checks and world boundaries on every graph read.
- Follow directed and conditional connections across entity types.
- Test empty worlds, disconnected subjects, repeated loading, and deleted subjects.
- Check large-world response times and client interaction.
- Verify keyboard navigation and readable connection details.

### 5. Deploy

- Release graph APIs before dependent client views.
- Verify query limits and required indexes.
- Browse a representative isolated world in the deployed client.
- Confirm partial results do not imply missing connections are absent from canon.

### 6. Maintain

- Monitor slow graph queries, oversized responses, and client rendering failures.
- Check navigation after graph schema changes.
- Keep performance examples for dense and disconnected graphs.
- Preserve accessible navigation as visual behavior changes.

## Epic 7: Reason About Consequences Within My World

**Implementation status — 2026-11-02:** Search and graph context exist. Dedicated consequence reasoning and deployed acceptance remain unfinished.

### 1. Define

**Stories:** US-032, US-033, US-034, US-035.

Writers explore possible changes without changing canon.
The system traces consequences through established facts and identifies missing premises.
A premise is a fact needed to support a conclusion.

**Acceptance:** Answers separate saved facts, inferred consequences, and unknown details.
Exploration changes no canon without explicit writer confirmation.

### 2. Design

- Mark hypothetical context explicitly for both chat and background research.
- Retrieve relevant relationship chains with sources and conditions.
- Explain each inferred consequence through supporting facts.
- Stop unsupported conclusions where required premises are missing.
- Offer deliberate confirmation before a proposed change becomes canon.
- Bound graph traversal and report incomplete evidence.

**Open decisions:** Hypothetical context boundaries, evidence path format, traversal limits, and confirmation behavior.

### 3. Build

1. Establish hypothetical handling across chat and research.
2. Add retrieval for relevant chains of relationships.
3. Add consequence explanations, source links, and missing-premise output.
4. Add explicit confirmation and safe saving through the canon workflow.

### 4. Test

- Explore a grain export ban using saved trade dependencies.
- Remove a required premise and verify the answer reports the gap.
- Test cycles, conflicting evidence, and conditional relationships.
- Verify graph contents stay unchanged during hypothetical discussion.
- Confirm only approved changes enter canon.

### 5. Deploy

- Enable reasoning after grounded retrieval and hypothetical safeguards pass.
- Run a deployed scenario and compare graph contents before and after discussion.
- Confirm evidence links work in the client.
- Preserve read-only exploration if confirmation writes fail.

### 6. Maintain

- Track unsupported conclusions and accidental canon writes.
- Recheck reasoning cases after model or retrieval changes.
- Review traversal limits as worlds grow.
- Keep missing-premise reporting useful rather than producing forced answers.

## Epic 8: Find Gaps and Choose What to Develop

**Implementation status — 2026-11-02:** Search foundations exist. Dedicated gap-review workflows and persistent open-question views remain unfinished.

### 1. Define

**Stories:** US-036, US-037, US-038, US-039.

Writers compare saved lore, notice uneven detail, and gather open questions.
They decide which gaps matter.
The system does not invent answers or impose a universal worldbuilding checklist.

**Acceptance:** Gap reports cite their basis and keep creative control with the writer.
Open questions remain available across sessions.

### 2. Design

- Compare related subjects using their saved details.
- Distinguish absent evidence from a confirmed absence in the fictional world.
- Present uneven detail as a question, not a contradiction or required correction.
- Save open questions with world scope and links to relevant subjects.
- Let writers revisit, answer, or dismiss questions.
- Keep generated questions outside canon.

**Open decisions:** Comparison scope, question states, duplicate question handling, and refresh behavior after lore changes.

### 3. Build

1. Add grounded comparisons using source-aware retrieval.
2. Add gap reports with evidence and clear limits.
3. Add open-question storage, APIs, and client views.
4. Connect answered questions to confirmed lore through existing save flows.

### 4. Test

- Compare detailed and sparse factions without inventing shared institutions.
- Distinguish unknown military structure from an explicitly absent military.
- Save, revisit, answer, and dismiss questions.
- Update lore and verify old questions do not become misleading.
- Verify reports and saved questions remain private to their world.

### 5. Deploy

- Apply question storage changes before enabling persistent review.
- Verify a comparison and saved-question workflow in an isolated deployed world.
- Confirm failed report generation leaves existing questions intact.
- Verify that question generation writes no canon facts.

### 6. Maintain

- Monitor failed reports, repeated questions, and outdated questions.
- Review whether comparisons help writers choose their next work.
- Recheck reports after prompt and model changes.
- Preserve writer dismissals and decisions when refreshing reports.

## Epic 9: Access and Recover My Account

**Implementation status — 2026-11-02:** Basic password signup, login, logout, and sessions exist. OAuth, email verification, recovery, and account linking remain unfinished.

### 1. Define

**Stories:** US-040, US-041, US-042, US-043, US-044.

Writers need secure sign-in and recovery without losing their existing worlds.

**Acceptance:** A writer can sign in, recover access, and link identities without exposing or duplicating their worlds.

### 2. Design

- Keep password sessions and add OAuth, which lets an external identity provider confirm identity.
- Use verified provider identity IDs; never merge accounts from matching email alone.
- Require authenticated confirmation for account linking and protect OAuth redirects against forged requests.
- Use short-lived, single-use verification and recovery tokens; avoid exposing whether an email has an account.

**Open decisions:** Provider selection, account linking rules, token expiry, and recovery for accounts without passwords.

### 3. Build

- Extend the existing account model with verified identities and recovery records.
- Build verification, OAuth, linking, and recovery flows with client feedback.

### 4. Test

- Test forged redirects, reused tokens, expired tokens, and duplicate provider identities.
- Verify account linking cannot take over another account or remove its last usable sign-in method.

### 5. Deploy

- Configure provider credentials and exact deployed callback addresses.
- Verify each supported sign-in and recovery flow using isolated accounts.

### 6. Maintain

- Monitor sign-in and recovery failures without recording tokens.
- Rotate credentials and review provider changes.

## Epic 10: Manage My Subscription and Payments

**Implementation status — 2026-11-02:** Stripe billing is required scope. No completed billing workflow is recorded.

### 1. Define

**Stories:** US-045, US-046, US-047, US-048, US-049, US-050.

Writers need clear pricing, secure payment, and predictable subscription changes.

**Acceptance:** Payment state reliably controls access, and writers can manage billing without losing their lore.

### 2. Design

- Use Stripe-hosted checkout and billing management; do not store payment card details.
- Verify signed billing events and persist received events before processing.
- Handle duplicate and out-of-order events safely; reconcile local state with Stripe.
- Keep subscription state separate from feature access rules.
- Preserve lore when payment fails; define read access, grace periods, and cancellation timing.

**Open decisions:** Prices, currencies, taxes, refunds, trial policy, plan changes, and payment-failure access rules.

### 3. Build

- Add customer, subscription, billing-event, and access records.
- Build checkout, billing management, event processing, reconciliation, and plan screens.

### 4. Test

- Test duplicate events, delayed events, failed payments, cancellation, and plan changes in Stripe test mode.
- Verify browser checkout redirects alone cannot grant paid access.

### 5. Deploy

- Verify test-mode workflows before enabling live credentials and event delivery.
- Publish prices and billing terms; verify reconciliation and rollback procedures.

### 6. Maintain

- Monitor event backlog, failed payments, and access mismatches.
- Reconcile subscriptions periodically and investigate charge disputes through controlled support access.

## Epic 11: Understand and Control Usage

**Implementation status — 2026-11-02:** Usage accounting and spending controls are required scope. No completed allowance workflow is recorded.

### 1. Define

**Stories:** US-051, US-052, US-053, US-054.

Writers need predictable allowances. Operators need enforceable spending controls.

**Acceptance:** Concurrent chat and background jobs respect limits, and writers see accurate usage and renewal information.

### 2. Design

- Record model usage by account, world, operation, and processing attempt.
- Reserve allowance before costly work and settle it against recorded usage.
- Make reservations safe across concurrent requests, retries, failures, and worker restarts.
- Include background research, embeddings, and retries in cost controls.
- Separate customer allowances from internal provider-cost budgets; never charge twice for the same billable action.

**Open decisions:** Allowance units, reset periods, retry charging, queued-work policy, and hard spending ceilings.

### 3. Build

- Build usage records, reservations, access checks, and worker budget enforcement.
- Add usage screens, renewal information, and clear limit responses.

### 4. Test

- Race concurrent requests against the final available allowance.
- Test failed calls, retries, expired reservations, renewal boundaries, and plan changes.

### 5. Deploy

- Validate accounting with isolated workloads before enforcing customer limits.
- Set internal spending ceilings and verify workers obey them.

### 6. Maintain

- Reconcile recorded usage and provider costs.
- Alert on abnormal spending, stuck reservations, and repeated limit bypass attempts.

## Epic 12: Receive Essential Service Emails

**Implementation status — 2026-11-02:** Service email delivery is required scope. No completed delivery workflow is recorded.

### 1. Define

**Stories:** US-055, US-056, US-057.

Essential emails help writers access accounts and understand service changes.

**Acceptance:** Essential messages arrive reliably, and failures remain visible and recoverable.

### 2. Design

- Use a service email provider and a durable outbound queue.
- Keep email content minimal; never include private lore or reusable credentials.
- Use message identity to prevent duplicate sends during retries.
- Track delivery failures, bounces, and suppressed addresses.
- Avoid duplicating billing notices already sent by Stripe.

**Open decisions:** Email provider, sender domain, required notices, and which system owns each billing notice.

### 3. Build

- Build templates, queued delivery, provider integration, and delivery status handling.
- Connect account and billing events to the required messages.

### 4. Test

- Test provider outages, duplicate processing, rejected addresses, and expired recovery links.
- Verify templates, links, sender identity, and private-content exclusion.

### 5. Deploy

- Verify sender-domain configuration and test real delivery to controlled addresses.
- Enable failure alerts and documented manual recovery.

### 6. Maintain

- Monitor queue age, bounces, complaints, and failed delivery.
- Review template accuracy and sender reputation.

## Epic 13: Operate and Recover the Service

**Implementation status — 2026-11-02:** Structured logging, cleanup retries, DBOS replay tests, and Ctrl+C shutdown exist. Hosting, alerts, backup recovery, SIGTERM behavior, and deployment acceptance remain.

### 1. Define

**Stories:** US-058, US-059, US-060, US-061.

The deployed service needs observable failures, tested recovery, and safe releases.

**Acceptance:** Operators detect failures and prove recovery within agreed downtime and data-loss targets.

### 2. Design

- Measure API, database, graph, worker, email, billing, and model-provider health.
- Keep private lore, credentials, and tokens out of routine logs.
- Define backup schedules and consistent recovery across Postgres and Memgraph.
- Define shutdown, migration, job recovery, readiness checks, and rollback behavior.

**Open decisions:** Hosting, alert ownership, acceptable downtime, acceptable data loss, and backup retention.

### 3. Build

- Add health checks, structured failure reporting, alerts, and release automation.
- Write incident, restore, migration, and rollback procedures.

### 4. Test

- Restore both stores into an isolated environment and verify source links and pending jobs.
- Test deployment shutdown, dependency outages, restart recovery, and rollback compatibility.

### 5. Deploy

- Prove restore and rollback procedures before accepting paid customers.
- Assign an alert recipient and verify deployed checks and notifications.

### 6. Maintain

- Run restore drills and review alert usefulness.
- Track capacity, dependency updates, and incidents with corrective actions.

## Epic 14: Protect and Manage My Data

**Implementation status — 2026-11-02:** Owner checks and world/conversation deletion exist. Exports, full account deletion, retention rules, and deletion during active research remain.

### 1. Define

**Stories:** US-062, US-063, US-064, US-065, US-066.

Writers own private worlds and need clear data controls and processing disclosures.

**Acceptance:** Writers can export and delete data, and published privacy behavior matches deployed handling.

### 2. Design

- Define export content, source links, format, and secure download expiry.
- Coordinate account deletion with subscriptions, jobs, graph data, identities, and stored exports.
- Document backup expiry and any retained billing records separately from active account data.
- Apply rate limits, secret management, least-privilege access, and dependency review.
- Publish actual data flows, provider processing, retention, and service terms.

**Open decisions:** Export format, retention periods, deletion confirmation, provider data handling, and launch policy requirements.

### 3. Build

- Build export and account-deletion jobs with visible progress and safe retries.
- Add access restrictions, rate limits, administrative audit records, and accurate policy pages.

### 4. Test

- Verify exports contain only the requesting writer’s data.
- Test deletion during active billing and research; verify retries cannot recreate removed data.
- Test abuse limits, privileged access controls, and secret exclusion from logs.

### 5. Deploy

- Review published policies against actual behavior and configured providers.
- Verify account deletion, export delivery, access controls, and secret configuration in the deployed service.

### 6. Maintain

- Review retained data, privileged access, and dependency risks.
- Keep policies current as providers and storage behavior change.

## Epic 15: Get Help and Resolve Service Problems

**Implementation status — 2026-11-02:** Research retry APIs exist for writers. Support intake, restricted operator tools, and audit workflows remain unfinished.

### 1. Define

**Stories:** US-067, US-068, US-069.

Writers need a support path. Operators need bounded tools to investigate and recover failed work.

**Acceptance:** Writers can report failures, and authorized operators can investigate and recover work without broad data access.

### 2. Design

- Attach safe operation references to errors and support reports.
- Provide authenticated tools for job inspection and approved retries.
- Restrict administrative actions, record their use, and hide private content by default.
- Keep support requests separate from automatic permission to inspect lore.

**Open decisions:** Support channel, response expectations, administrative roles, and rules for authorizing private-content access.

### 3. Build

- Add support links, useful error references, and operational lookup tools.
- Add controlled retry actions and administrative audit records.

### 4. Test

- Verify support references expose no private content.
- Test role restrictions, retry safety, and audit records.

### 5. Deploy

- Publish the support channel and verify report delivery.
- Limit administrative access and rehearse recovery using an isolated failed job.

### 6. Maintain

- Track unresolved reports and recurring failures.
- Convert repeated incidents into regression tests and product fixes.
