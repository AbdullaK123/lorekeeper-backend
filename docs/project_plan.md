# Lorekeeper — Project Plan

## US-043 closed (backend) — 2026-11-12

Unlinking cut. Linking backend done and tested. Next backend priority returns to the open plan items: billing and usage, concurrent-edit protection, stale-embedding repair.

## US-043 next steps — 2026-11-12

Linking done and tested (33/33). Next: decide if logged-in /google should always link; then build unlink.

## Session lookup fix — 2026-11-12

Google start and callback use Option<UserId>; the hidden-error test passes. 32/33 link tests pass. Overwrite rule still open.

## US-043 next steps — 2026-11-12

Fix: use Option<UserId> in Google start/callback so session-lookup errors surface. Decide the overwrite rule. Then unlinking.

## US-043 link test evidence — 2026-11-12

Linking service path has 16 passing tests. Next: decide the overwrite rule and whether logged-in /google always links.

## Google account linking backend — 2026-11-12

US-043 linking path is implemented in the backend but not accepted. No link tests exist yet.
Plain Google login tests pass (9/9). Callback tests need DBOS system tables in the new QA database.
Open design points: logged-in /google always links; existing google_sub is overwritten without warning.

## Next backend slice after Google sign-in — 2026-11-10

Google sign-in has persisted-user/session evidence. Account linking remains separate unfinished work under US-043.
Recommend authenticated Google linking for existing password accounts. Current email collisions reject login rather than link accounts.
Link only after the user proves both identities: an existing local session and verified Google login. Preserve the existing user ID and worlds.
First design slice: define the link request/callback behavior and transaction boundary before implementation.
Billing and usage remain required later work. All four PM docs reviewed; no code changes or tests occur and no full epic is newly accepted.

## SQLx Google field metadata refreshed — 2026-11-10

Local Postgres is healthy and users.google_sub exists. The previously missing .sqlx cache is regenerated against app_db.
Online all-target compilation through cargo sqlx prepare passes. All eight user-returning query metadata entries include google_sub.
An offline library check also passes with existing warnings. The reported missing-field error is not reproduced by the compiler.
Rust Analyzer still needs a workspace/server restart to discard its previous macro expansion; editor refresh is not verified from this session.
No application logic edits or tests occur. All four PM docs reviewed; scope and acceptance remain unchanged.

## Bootstrap application migrations — 2026-11-10

Bootstrap now runs embedded SQLx migrations before repository/service/worker creation. Migration failure stops startup and emits an error log.
Docker's API build stage copies migrations for compile-time embedding. README startup instructions reflect automatic migration.
Offline library checks are blocked by missing .sqlx query metadata. The first incremental check also panics; a non-incremental retry reports missing metadata without that panic.
No missing cache files are restored. Whitespace checks pass. Runtime migration execution, tests, and Docker rebuild are not verified this turn.
All four PM docs reviewed. Product scope and acceptance remain unchanged; refresh query metadata before build verification.

## Tests moved outside source — 2026-11-10

Moved five Rust unit-test modules into tests/unit, including the previously inline playground tests.
Test-only path declarations preserve private access and existing test names. No production API visibility changes occur.
Moved the playground client test into tests/playground and updated its npm command. AGENTS.md now records this location rule.
Rust library tests compile: 13 pass, one existing duplicate-row ranking failure, 32 ignored storage/provider tests.
All six playground client tests pass. Source scan finds no test functions under src or playground/src; git diff --check passes.
No live storage/provider tests run. All four PM docs reviewed; product scope and acceptance remain unchanged.

## Missing Google field diagnostic checked — 2026-11-10

User repository queries select/return all columns; prepared user query metadata includes google_sub.
Offline library cargo check passes with existing warnings. Online check fails because localhost:5433 refuses connections.
Docker Compose lists no running app-db container. Rust Analyzer settings require live SQLx checks against that database.
The reported missing-field diagnostic is not reproduced by the current offline compiler check; stale IDE macro expansion remains possible.
Next: start the local database, then reload the Rust Analyzer workspace. No application edits or tests occur.
All four PM docs reviewed. Delivery scope and acceptance remain unchanged.

## Google login database confirmation — 2026-11-10

Read-only inspection of local app_db finds one Google-linked user: verified, passwordless, with one active session.
This confirms persisted identity/session creation after the user's sign-in. Browser cookie delivery and /auth/me are not checked this turn.
No personal data or token values are printed. No application edits, automated tests, or provider calls occur.
All four PM docs reviewed. Product scope stays unchanged; full browser acceptance still needs its remaining checks.

## Google consent branding diagnosis — 2026-11-10

The local Google start route returns 303. Its client ID matches the current .env; stale runtime credentials are not the cause at this endpoint.
Its callback is http://localhost:8000/api/auth/google/callback. No redirect was followed and no credentials were printed.
Google branding belongs to the Cloud project, shared across its clients. Nexus Writer branding suggests the configured client uses that project's branding.
Use a separate Nexlore Cloud project/client for separate branding, then update credentials and recreate the API container.
Google Console project ownership/branding is not inspected. No application edits or tests occur; live acceptance stays pending.
All four PM docs reviewed. Scope and story acceptance remain unchanged.

## Playground Google sign-in — 2026-11-10

Added Google sign-in through a separate browser window. The playground checks its session and closes the window after login.
Cancellation, blocked pop-ups, session-check errors, and a ten-minute limit have visible feedback. Unmount stops polling.
The backend callback keeps its JSON response. The playground and API must use the same hostname for cookies.
Build and six existing playground tests pass. Four Playwright UI checks pass with controlled provider/session responses.
The running API returns 404 for Google start, so this run does not establish live backend/provider acceptance.
Evidence: tests/artifacts/playwright-oauth-2026-11-10/report.json and screenshots. Live Google sign-in remains pending.
All four PM docs reviewed. Product scope and epic acceptance remain unchanged.

## OAuth callback cookie cleanup fixed — 2026-11-10

The callback collects its result inside an async block. The outer handler removes oauth_state on success and handler errors.
Successful login still adds the session cookie. Error responses keep their original status.
All three local callback HTTP tests pass, including the previously failing missing-attempt cleanup test.
Formatting and git diff --check pass. No live Google sign-ins, emails, or paid calls occur.
Query extractor rejection occurs before handler cleanup and remains outside this change.
All four PM docs reviewed. Product scope stays unchanged; live Google/browser acceptance remains pending.

## OAuth cookie cleanup cause — 2026-11-10

Inspection confirms callback service errors return through `?` before cookie removal.
Proposed fix: collect the callback result, then return cookie removal with either response.
This covers handler errors; query extractor rejection occurs before the handler runs.
No application code changes or tests occur this turn. Scope and acceptance remain unchanged.
The failing callback cleanup test remains the next check after implementation.

## Google OAuth regression tests — 2026-11-10

Google login now has repository, local-provider, and callback failure-path evidence. Acceptance remains blocked by callback error-cookie cleanup and pending live Google/browser verification.
Executed results: 64 pass, two fail. One failure is the newly demonstrated stale OAuth state cookie after a callback service error; the other is the existing retrieval-ranking test.
No production logic fixes, real Google sign-ins, emails, paid calls, or deployments occur. Formatting/whitespace checks pass and QA user fixtures are removed.
Evidence: tests/artifacts/google-oauth-tests-2026-11-10/summary.md. All four PM docs reviewed; product scope stays unchanged.

## OAuth cancellation branch implemented — 2026-11-10

OAuthParams now accepts optional code/error values and requires state. After browser-state validation, only code-present/error-absent proceeds to exchange.
Cancellation or invalid code/error combinations clear oauth_state and return a generic authentication error response. Provider text is not echoed or logged.
The callback now returns a concrete Response for both success and cancellation, and has skip_all tracing.
Applied the authored google_sub migration to isolated QA Postgres, refreshed SQLx metadata, and compile-checked all targets. Final offline all-target check passes with existing warnings.
Formatting and git diff --check pass. No tests, Google discovery/sign-in, real emails, or paid calls occur.
Other early error paths still return via ? and do not yet share cookie cleanup. Integrated OAuth runtime tests and live acceptance remain pending.
All four PM docs reviewed. Scope and product acceptance stay unchanged.

## OAuth cancellation handling guidance — 2026-11-10

Propose optional code/error fields while keeping state required. Validate state/cookie before interpreting the result.
Accept only code-present/error-absent for exchange. Denial or invalid combinations return a generic auth error response paired with cookie removal; never echo provider error descriptions.
Use a concrete axum Response return so cancellation and successful JSON can each call into_response. Extract code before calling the service.
Other early errors currently using ? still require unified cookie cleanup later; this guidance does not claim all error paths are covered.
All four PM docs reviewed. Teaching only: no application edits, compilation, provider calls, or tests. Verification remains pending.

## Callback route registration confirmed — 2026-11-10

Inspection confirms the cookie helper and all call sites remain present. The user has registered GET /google/callback.
Next: support provider denial callbacks where error is present instead of code, and ensure temporary cookie cleanup on failure responses.
Schema/cache refresh, all-target compilation, and OAuth integration tests remain pending.
All four PM docs reviewed. No application edits or tests occur this turn. Runtime sign-in acceptance stays open.

## Cookie helper reapplied and verified on disk — 2026-11-10

User reports the prior edits are missing. Inspection confirms the controller again contains inline cookie builders and an unfinished callback response; the helper is absent.
Reapplied the helper, its login/logout/Google call sites, the Deserialize import, and the callback's cookie/JSON success response.
Read the saved file after formatting: auth_cookie exists at line 17; one Cookie::build remains inside the helper; five call sites use it.
Formatting and git diff --check pass. No compilation or tests run. Cause of the missing prior edits is not established.
All four PM docs reviewed. Callback registration, error handling, and integrated verification remain pending; acceptance is unchanged.

## Shared auth-cookie helper added — 2026-11-10

Added private AppState::auth_cookie(name, value) in the auth controller file. The helper uses existing is_prod state and configures Path=/, HttpOnly, SameSite=Lax, and production Secure.
Password login, logout removal, Google start, and Google callback success now use the helper. Lifetimes stay explicit at the call sites: stored session expiry versus ten-minute OAuth state.
Filled the callback's success cookie/JSON response using the committed user/token/expiry; it clears oauth_state and does not insert another session.
Added the missing Deserialize import. Formatting and git diff --check pass. No compilation, tests, migrations, provider calls, or SQLx refresh occur.
Callback route registration, provider-denial/error handling, updated schema/cache, and integrated verification remain pending.
All four PM docs reviewed. Product scope and acceptance remain unchanged.

## Google completion uses atomic login — 2026-11-10

Updated complete_google_login to generate a raw session token/hash after Google validation and call UserRepository::login_with_google for the atomic user/session operation.
It now returns (User, raw token, database session expiry). It no longer signs up a new user independently of session insertion.
Existing subject lookup retains the prior behavior and supplies saved profile inputs; only new subjects require a present verified email. The parent transaction performs the authoritative user resolution.
Added skip_all service tracing and a safe success event after repository commit. Updated callback destructuring for the new result; its cookie/response body remains unfinished.
Formatting and git diff --check pass. No compilation, SQLx refresh, migrations, provider calls, or tests occur this turn.
Next: build the callback session cookie from the returned token/expiry, remove oauth_state, return the response, and register/verify the route.
All four PM docs reviewed. Product scope and acceptance remain unchanged.

## Google user/session repository transaction added — 2026-11-10

Added UserRepository::login_with_google(google_sub, username, email, session_token_hash), returning the committed (User, SessionRow).
The method resolves an existing subject or inserts a verified passwordless user, then inserts its session on the same transaction connection. Errors explicitly roll back; success logs only after commit.
Existing subject profiles remain unchanged. The insertion conflict target handles repeated subject creation; existing-email collisions retain the Conflict response without linking accounts.
SessionRepository now exposes a connection-aware insert shared by normal creation and the parent's transaction. Raw session secrets remain a service responsibility; the repo takes only a hash.
Added three database tests covering returning-user reuse, failed session insertion rolling back user creation, and rejection of existing-email collisions.
Formatting and git diff --check pass. Tests and compilation are not run while surrounding callback code is unfinished; new SQLx query metadata and google_sub schema verification remain pending.
Next: AuthService generates the session token/hash and calls this parent method after Google verification, returning user/raw token/expiry for the cookie response.
All four PM docs reviewed. No service/controller wiring, migration execution, provider calls, or acceptance completion occur this turn.

## Google first-login transaction boundary — 2026-11-10

User identifies that first-time Google user creation and local session insertion should share a database transaction.
Current signup and session-create queries independently use the pool, so this boundary is not implemented.
Revise the proposed callback flow: AuthService verifies Google outside the transaction and generates a session secret/hash; UserRepository resolves/creates the user and inserts the session on one transaction connection, then commits.
Return committed user/session expiry to the service/controller; set the raw-token cookie only after commit. The session child needs a connection-aware insertion path.
If session insertion fails, roll back new user creation. Existing-user login can use the same parent operation. Google token exchange and attempt consumption are not rolled back by this local transaction.
All four PM docs reviewed. Design discussion only: no application edits or tests. Previous separate controller create_session guidance is superseded for Google completion.

## Callback session response next step — 2026-11-10

The callback currently stops after obtaining User from complete_google_login.
Next reuse the password-login session creation and cookie policy, remove oauth_state at Path=/, and return the updated jar plus UserResponse JSON as a backend-only interim response.
Then register GET /google/callback. A frontend success redirect is deferred until its destination is defined.
All four PM docs reviewed. No application edits or tests occur. Denial handling and integrated compilation/testing remain pending.

## OAuth constructor wiring fixed — 2026-11-10

Bootstrap now passes the retained HTTP client alongside GoogleClient to AuthService::new.
Fixed the OAuth factory return type to refer to reqwest::Client, avoiding the missing Client import.
Updated the auth repository test constructor with local synthetic provider metadata and a no-redirect HTTP client; it performs no Google discovery.
The callback already uses params.code in the user's current source, so no argument edit is needed there.
Formatting and git diff --check pass. No compilation, tests, migrations, provider requests, or SQLx refresh occur while callback response handling remains unfinished.
All four PM docs reviewed. Next: create the callback session/cookies and response, then register the route and handle denial. Sign-in acceptance remains pending.

## Remaining Google integration checklist — 2026-11-10

Source inspection identifies immediate wiring blockers: callback passes undefined code instead of params.code; bootstrap omits the new HTTP-client constructor argument; repository test constructor also needs updating.
Callback still needs local session creation, secure session cookie, temporary-state cookie removal, a response, and GET /google/callback registration.
OAuthParams still requires code and cannot represent Google denial error responses. Add callback/service skip_all tracing before testing.
Apply google_sub migration and refresh affected SQLx metadata. Compile all targets and add local-provider tests for state, replay/expiry, token validation, identity resolution, and sessions before live acceptance.
Frontend success destination remains a separate decision. No full Google sign-in has been verified.
All four PM docs reviewed. No application edits, compilation, database operations, provider calls, or tests occur this turn.

## Explicit signup verification flag added — 2026-11-10

Repository signup now takes verified: bool and inserts it with the user's password hash and Google subject in one statement.
Password signup passes Some(password_hash), None, false. Google signup passes None, Some(subject), true after its verified-email claim guard.
Updated both source call sites, including the stale password-signup signature. Public signup payload remains unchanged; verification is selected by server code.
Formatting and git diff --check pass. No compilation, SQLx cache refresh, migrations, provider calls, or tests occur while callback wiring remains unfinished.
All four PM docs reviewed. Next: finish callback/session response and verify the integrated flow. Product scope and acceptance stay unchanged.

## Google user resolution review — 2026-11-10

Inspection confirms complete_google_login returns existing subject matches or creates a passwordless user from claims after requiring a verified email.
Observed mismatch: repository signup still omits verified, so new Google users retain the database default false despite the verified claim check.
Next correction: pass an explicit verified flag into repository signup and insert it atomically. Password signup passes false; Google signup passes true only after the existing claim check.
Email uniqueness still rejects existing-account collisions through the current repository mapping. Callback completion and tests remain pending.
All four PM docs reviewed. No application edits, compilation, provider calls, migrations, or tests occur. Google sign-in is not yet complete.

## Reading Google signup claims — 2026-11-10

Checked installed 4.1.0 claim accessors. email() returns an optional email; email_verified() returns Option<bool>; name() returns an optional localized claim whose get(None) reads the default-language name.
Guidance extracts an owned email, requires email_verified=true for new Google signup, and uses the default display name with a Writer fallback as the initial username.
Display name is not the identity key; continue using verified subject for account lookup. Existing subject matches need not be recreated.
All four PM docs reviewed. No application edits, compilation, provider calls, or tests occur. Signup integration remains pending.

## Signup supports Google subjects — 2026-11-10

Changed UserRepository::sign_up to accept Option<String> for password_hash and google_sub, and insert both through bound SQL parameters.
Updated the existing password signup caller to pass Some(password_hash), None. Public signup request fields stay unchanged; clients cannot submit arbitrary Google subjects through signup.
Google callback code can pass None for password_hash and Some(verified_claim_subject) for google_sub. The existing verified default is unchanged; this does not treat subject presence as email verification.
Formatting and git diff --check pass. No compilation, SQLx metadata refresh, migrations, or tests occur while the callback flow remains unfinished.
Next: complete verified-email handling and user resolution in the callback service before creating the session.
All four PM docs reviewed. Product scope and acceptance remain unchanged.

## Google user resolution flow discussion — 2026-11-10

User proposes returning an existing google_sub match or creating a passwordless user for a new subject. This matches the nullable password_hash schema.
Creation must use verified ID-token claims: store google_sub, required email, username, and NULL password_hash. Only set verified=true when Google's email_verified claim is true.
Recommended initial policy: require a present verified email for Google signup and reject existing-email collisions for explicit linking later; do not silently attach a new subject.
Service should return User; controller then creates the normal local session. No implementation or acceptance of account-linking behavior is recorded yet.
All four PM docs reviewed. No application edits, migrations, compilation, provider calls, or tests occur.

## Google user lookup added — 2026-11-10

User model now has google_sub: Option<String>. Added UserRepository::get_user_by_google_sub(&str), using a bound query and fetch_optional to return Result<Option<User>, InfrastructureError>.
Tracing skips all arguments and does not log the provider subject or returned user.
Formatting and git diff --check pass. No compilation, tests, migration execution, or SQLx metadata refresh occur; surrounding OAuth callback/service work remains unfinished.
The new query needs database/SQLx verification once the schema and unfinished flow are ready. Next service step is resolving verified claims.subject() through this lookup.
All four PM docs reviewed. Google sign-in and product acceptance remain pending.

## Google subject uniqueness follow-up — 2026-11-10

Inspection confirms the new google_sub declaration is nullable and UNIQUE.
User model still needs google_sub: Option<String> to match the added column returned by existing SELECT/RETURNING * queries after migration.
Next repository operation is get_user_by_google_sub returning Option<User>. Keep Google subject internal; public UserResponse need not expose it.
All four PM docs reviewed. No application edits, migration runs, compilation, or tests occur. Identity resolution remains incomplete.

## Google subject migration review — 2026-11-10

The migration adds a nullable TEXT google_sub column, preserving existing password users.
UNIQUE is missing. Add it so one non-null Google subject cannot belong to multiple local users. PostgreSQL's default unique semantics allow multiple NULL values.
All four PM docs reviewed. Inspection only: no application or migration edits, SQL execution, compilation, or tests. Identity mapping remains pending.

## Google identity mapping next step — 2026-11-10

Inspection confirms code exchange, PKCE verification input, and ID-token claims validation are written. The service still lacks its final return.
Next proposed schema slice for the selected Google-only scope: nullable UNIQUE users.google_sub to map the verified Google subject to the local user.
Existing google_sub resolves the account. New identities may create passwordless users; an existing email without that mapping requires an explicit account-linking policy, not silent merge.
Propose rejecting an email collision and requiring existing-account login for later linking. This policy is not accepted or implemented yet.
The eventual service return should be User, enabling the controller's existing custom-session creation.
All four PM docs reviewed. No application edits, migration changes, compilation, provider calls, or tests. Sign-in remains incomplete.

## Google code exchange guidance — 2026-11-10

Checked installed openidconnect 4.1.0 async example. complete_google_login now accepts code but still returns saved nonce/verifier without exchanging it.
Guidance uses exchange_code(AuthorizationCode), set_pkce_verifier with the saved raw verifier, and request_async with a retained no-redirect HTTP client.
Then require id_token and call claims with the configured ID-token verifier and saved Nonce. Only verified claims can drive identity lookup.
AuthService currently lacks the reusable HTTP client field; constructor/bootstrap wiring is a prerequisite. Provider identity mapping and final user return type remain unimplemented.
All four PM docs reviewed. No application edits, network requests, compilation, or tests occur. Sign-in remains incomplete.

## Service attempt consumption review — 2026-11-10

Inspection confirms complete_google_login atomically consumes the attempt and maps absent/expired/replayed state to AuthError.
The draft returns nonce/verifier only and does not accept Google's authorization code. It is not yet a completed login operation.
Next: accept code too, retain nonce/verifier inside AuthService, exchange code with the saved verifier using the no-redirect async HTTP client, then validate the ID token with the saved nonce.
All four PM docs reviewed. No application edits, compilation, provider calls, or tests occur. Full callback and identity mapping remain pending.

## Callback browser-state check reviewed — 2026-11-10

Inspection confirms missing-cookie rejection and comparison of cookie/query state hashes occur before any attempt consumption.
Next service slice: complete_google_login receives authorization code and checked state hash, consumes the unexpired attempt through UserRepository, and maps None to AuthError.
Keep saved nonce/verifier and token exchange inside AuthService. Controller retains cookie/query handling.
All four PM docs reviewed. No application edits or tests occur. Callback return, registration, denial handling, exchange, and identity mapping remain pending.

## Callback draft state-check guidance — 2026-11-10

The draft reads oauth_state and hashes returned state but does not compare them. Cookie presence alone does not bind the callback to its browser.
Next compare SHA-256 hashes of returned state and cookie value, rejecting mismatches before consuming any saved attempt.
The ok_or_else call passes an error value instead of a closure; use ok_or for the fixed error, or wrap it in a closure.
The callback remains unregistered/incomplete and requires code-only OAuthParams, so provider-denial handling also remains pending.
All four PM docs reviewed. No application edits, compilation, or tests occur. Sign-in remains unverified.

## Next OAuth slice: callback validation — 2026-11-10

Next implement GET /auth/google/callback inputs and browser binding before any token exchange.
Read returned state/code (or provider denial), require matching oauth_state cookie using a timing-safe comparison, then atomically consume the hashed unexpired attempt.
Reject absent/mismatched/expired/reused attempts and clear the temporary cookie in the callback response. Preserve nonce/verifier for the next token exchange step.
No callback route exists at review time. All four PM docs reviewed; no application changes or tests occur. Full Google sign-in remains pending.

## Google login tracing added — 2026-11-10

Confirmed the start route returns the updated ten-minute oauth_state cookie and Google redirect.
Added skip_all spans to the handler, service, discovery factory, and attempt child queries. Events record provider, setup stage, stored attempts, prepared redirects, and consume match status.
No credentials, state/hashes, nonce, PKCE verifier, URLs, or return values are logged. Updated docs/logging.md.
Compile checking exposed an ambiguous child import and uncached new queries. Fixed the import, applied the authored OAuth-attempt migration only to QA, and refreshed SQLx metadata.
Online library compilation and the final offline library check pass with existing warnings. Formatting and git diff --check pass.
No tests, Google discovery requests, real sign-ins, emails, or paid calls occur. Full test-target compilation is not checked this turn.
Next: callback browser-state validation and atomic attempt consumption, followed by token exchange and identity mapping.
All four PM docs reviewed. Google sign-in remains incomplete; product scope and acceptance stay unchanged.

## Google start handler draft review — 2026-11-10

The draft calls AuthService and builds oauth_state with HttpOnly, Path=/, SameSite=Lax, and production Secure.
Remaining: a ten-minute cookie max_age, returning updated CookieJar with Redirect::to, registering GET /google in the auth router, and a skip_all tracing span.
The current handler has no return expression. No callback implementation exists yet.
All four PM docs reviewed. No application edits, compilation, migrations, provider requests, or tests occur. Google flow remains incomplete.

## Google login-start persistence review — 2026-11-10

Inspection confirms begin_google_login hashes generated state, saves nonce/verifier with the attempt, then returns URL/raw state after a successful insert.
Next: GET /auth/google sets a ten-minute HttpOnly oauth_state cookie at Path=/ with SameSite=Lax and production Secure, then returns that updated jar plus a redirect.
The callback must compare query state with the cookie before consuming the unexpired attempt. This is not implemented yet.
All four PM docs reviewed. No application edits, compilation, database execution, or tests occur; sign-in remains incomplete.

## Persisting Google login attempts guidance — 2026-11-10

begin_google_login now declares a URL/state pair return but still stops after URL generation.
Next: SHA-256 hash state.secret(), save that hash plus nonce.secret() and verifier.secret() through create_login_attempt, then return URL and raw state only after persistence succeeds.
Keep raw nonce/verifier in the database for callback validation/exchange; do not log these values or the generated URL.
All four PM docs reviewed. Guidance only: no application edits, migration execution, compilation, or tests. Controller redirect remains pending.

## Login-attempt forwarding methods added — 2026-11-10

Added UserRepository::create_login_attempt and consume_login_attempt, forwarding to the existing child with unchanged result types.
Both spans use skip_all so state hashes, nonces, verifiers, and return values are not logged.
Changed file is formatted and git diff --check passes. No compilation, tests, migrations, or provider calls occur for these mechanical wrappers.
Next: persist the generated attempt from AuthService and return URL/raw state for the controller.
All four PM docs reviewed. Google login remains incomplete; scope and acceptance stay unchanged.

## Login-attempt child construction confirmed — 2026-11-10

Inspection confirms UserRepository holds and constructs LoginAttemptsRepository; the child derives Debug.
Forwarding methods are still absent. Next: expose create_login_attempt returning LoginAttemptRow and consume_login_attempt returning Option<LoginAttemptRow> through the parent.
All four PM docs reviewed. No application edits, compilation, or tests occur. AuthService persistence remains pending.

## Optional attempt consumption confirmed — 2026-11-10

Inspection confirms LoginAttemptRow spelling is corrected and consume now uses fetch_optional with an Option return.
Next: UserRepository holds/constructs LoginAttemptsRepository and forwards create/consume operations. Child needs Debug to match the parent's existing derive.
All four PM docs reviewed. No application edits, compilation, migration runs, or tests. OAuth integration remains pending.

## Login-attempt repository review — 2026-11-10

Model and child queries now exist. consume uses atomic DELETE RETURNING with an expiry predicate.
Issue: fetch_one turns expected expired/missing/replayed attempts into RowNotFound database errors. Recommend fetch_optional and Result<Option<LoginAttemptRow>, InfrastructureError> for service-level authentication rejection.
Model name currently has a typo: LoginAttempRow. The parent declares the module but has not constructed/held the child or exposed its operations.
All four PM docs reviewed. Inspection only: no application edits, compilation, database execution, or tests. Callback wiring remains pending.

## OAuth attempt migration typo corrected — 2026-11-10

Inspection confirms expires_at now uses TIMESTAMPTZ. The previously identified SQL type typo is fixed in source.
Next: add the attempt row model and child repository with create and atomic consume operations; consume deletes and returns only unexpired matching rows.
All four PM docs reviewed. No application edits, migration execution, compilation, or tests occur. Database verification and OAuth flow remain pending.

## OAuth attempt migration review — 2026-11-10

Reviewed migrations/20261110065905_oauth_attempts.sql. Fields, nullability, primary key, and ten-minute expiry match the proposed attempt store.
One SQL type typo remains: TIMESTAMPZ must be TIMESTAMPTZ.
All four PM docs reviewed. No application/migration edits, database execution, or tests occur. Migration is not verified or applied by this turn.

## Google login-attempt migration scope — 2026-11-10

Inspection confirms begin_google_login generates URL/state/nonce/PKCE values but has no return or persistence yet.
Next migration proposal: google_login_attempts(state_hash TEXT primary key, nonce TEXT not null, pkce_verifier TEXT not null, expires_at TIMESTAMPTZ not null default NOW()+10 minutes).
No user_id is required because identity is unknown at login start. Store the actual verifier for exchange; a hash cannot replace it.
The controller needs raw state for its temporary browser cookie, so returning only the URL is insufficient. Return URL/state after saving the attempt.
Consume matching unexpired rows atomically during callback. Google identity mapping is a separate later schema task.
All four PM docs reviewed. No migration/application edits, database operations, compilation, or tests occur. Sign-in remains incomplete.

## Google login-start design proposal — 2026-11-10

Split work into authorization URL generation, temporary login-attempt storage, and a controller redirect.
AuthService generates URL, state, nonce, and PKCE verifier using the configured Google client and email/profile scopes.
Proposed persistence: a short-lived Google login-attempt row with state hash, nonce, PKCE verifier, and expiry. The browser gets a temporary HttpOnly state cookie at Path=/ with Lax/production Secure policy.
The callback must match query state to browser state, atomically consume the unexpired attempt, then use its verifier and nonce for token exchange/validation.
This browser binding is required; a state value stored only in the database is insufficient to connect the callback to the initiating browser.
First implementation slice is just URL/value generation in AuthService; storage and routes follow before any live login flow is usable.
All four PM docs reviewed. No application edits, migrations, network requests, or tests occur. This storage design is proposed, not implemented or accepted.

## OAuth startup handling follow-up — 2026-11-10

Inspection confirms bootstrap now awaits Google client discovery and handles failure with an explicit startup expect.
The reviewed tuple-return mismatch is resolved in source. No compile or live discovery verification occurs.
Next slice: build the Google authorization URL and retain browser-bound temporary login state for the callback.
All four PM docs reviewed. No application edits or tests occur. Google sign-in acceptance remains pending.

## Bootstrap OAuth wiring review — 2026-11-10

AuthService now accepts and stores a completed GoogleClient; bootstrap awaits its factory before construction.
One compile blocker remains: bootstrap returns a tuple but uses ? on discovery. Match existing startup behavior with an explicit expect, or deliberately change the full startup error return chain.
Recommend the local expect for this step; do not expand the refactor into unrelated startup APIs.
All four PM docs reviewed. No application edits, compilation, discovery calls, or tests occur. Wiring is not yet verified.

## Async Google factory type mismatch — 2026-11-10

AuthService::new assigns create_core_client directly to a GoogleClient field. The async call returns a Future yielding Result<GoogleClient, ServiceError>.
Explain that await obtains the Result and error handling obtains the client. The synchronous constructor cannot await.
Keep discovery in async bootstrap and pass the completed GoogleClient into the constructor. The factory expects &Settings, not an owned Arc clone.
Bootstrap still calls the old constructor signature and must be aligned when the user completes this change.
All four PM docs reviewed. No application edits, compilation, or tests occur. Integration remains incomplete.

## Google client factory review — 2026-11-10

Inspection confirms the factory now discovers Google metadata, constructs the correctly typed client with credentials, and sets the callback URL.
Next: call the factory once during bootstrap and retain the client for AuthService. Retain/reuse a no-redirect async HTTP client for later token exchanges too.
No OAuth start/callback routes or login-attempt storage exist yet. Callback registration must still match the configured URL.
All four PM docs reviewed. No application edits, compilation, live discovery, or tests occur. Setup is reviewed by inspection only; acceptance stays pending.

## Constructing the discovered Google client — 2026-11-10

Inspection confirms async HTTP client and provider discovery are written. The function still lacks its return value.
Next guidance constructs CoreClient from metadata and owned ClientId/ClientSecret, then sets the exact Google callback RedirectUrl.
Installed 4.1.0 source confirms endpoint states: EndpointSet, three EndpointNotSet, then two EndpointMaybeSet. A GoogleClient alias represents that return type.
Credentials are cloned once because the client owns them. Callback URL derives from API_BASE_URL plus /auth/google/callback and must match registration.
All four PM docs reviewed. No application edits, discovery requests, compilation, or tests occur. Google setup remains unverified.

## Google discovery implementation guidance — 2026-11-10

openidconnect 4.1.0 is now present and oauth.rs contains an empty create_core_client stub.
First slice: make setup async, build the re-exported reqwest client with redirects disabled, and call CoreProviderMetadata::discover_async for https://accounts.google.com.
Map setup/discovery errors to ServiceError; never log Settings or secrets. Construct the client once during startup.
The plain CoreClient return annotation uses unset endpoint defaults; final client construction will need the discovered endpoint type reflected in its return type.
Reviewed installed crate source after the documentation fetch failed. No application edits, network discovery requests, or tests occur this turn.
All four PM docs reviewed. Client construction and Google sign-in acceptance remain pending.

## OpenID Connect crate walkthrough — 2026-11-10

Explained openidconnect 4.1.0 using its official documentation: discover provider metadata, construct CoreClient, build authorization URLs, exchange callback codes asynchronously, then verify ID-token claims.
The application still owns browser-bound login-attempt state, state/nonce/PKCE persistence, user identity mapping, and local session creation.
No dependency is installed yet. First implementation slice remains Google discovery/client construction at startup.
Reviewed all four PM docs. Teaching only: no application edits, tests, or provider changes. Delivery status stays unchanged.

## Credentials wrapper removed — 2026-11-10

User removes GoogleOAuthCredentials and its getter, exposing client_id/client_secret on Settings directly.
This avoids a one-use borrowed wrapper and lifetime plumbing. Setup functions can still accept &str arguments for only the values they need.
All four PM docs reviewed. Inspection only: no application edits or tests occur. Google setup remains pending; scope and acceptance stay unchanged.

## Borrowed credentials tradeoffs — 2026-11-10

Inspection confirms the getter now returns GoogleOAuthCredentials<'_> tied to Settings.
The wrapper narrows a function's inputs to Google credentials, groups related fields, and borrows existing strings without allocating copies.
It does not encrypt or redact secrets. Its references cannot outlive Settings; an owned OpenID client can copy the required values at startup.
Teaching only: no application changes or tests occur. All four PM docs reviewed; delivery status and scope stay unchanged.

## Borrowed Google credentials review — 2026-11-10

Settings now declares client_id/client_secret and returns a borrowed GoogleOAuthCredentials view.
The getter's independent lifetime parameter is not tied to &self. Use get_oauth_creds(&self) -> GoogleOAuthCredentials<'_> so the return borrows Settings.
No client construction or OAuth routes exist yet. Environment values are not read or displayed during this review.
Reviewed all four PM docs. No application edits, compilation, or tests occur. Google integration and acceptance remain pending.

## Google sign-in selected — 2026-11-10

User selects Google as the first OAuth sign-in provider for US-040.
Proposed Rust integration uses openidconnect and Google's OpenID Connect authorization-code server flow.
Routes: GET /auth/google starts the browser redirect; GET /auth/google/callback validates the login attempt, exchanges the code, verifies the ID token, resolves the provider identity, and creates the existing app session.
Use Google's stable sub claim for identity, not email as the identity key. Account-linking rules remain undecided.
Login attempts need browser-bound, expiring state/nonce and PKCE data. Implementation must validate returned tokens through the library.
First setup step: create a Google Web application OAuth client and register the exact callback URL. Keep its secret in settings, never source or logs.
Checked official Google documentation and openidconnect crate documentation. No dependency additions, application edits, tests, or Google configuration changes occur.
Reviewed all four PM docs. Provider choice is settled; implementation and epic acceptance remain pending.

## Next feature proposal: OAuth sign-in — 2026-11-08

Password recovery, verification, and custom-session HTTP checks now have evidence. Frontend confirmation pages stay deferred.
Propose OAuth sign-in (US-040, Epic 9) as the next backend feature. No provider is selected yet.
First agree the provider and account-linking behavior, then design provider identity storage and login/callback routes using the existing session service.
This is a priority proposal, not implementation approval or epic completion.
Reviewed all four PM docs and Epic 9 scope. No application changes or tests occur this turn.

## Verification HTTP workflow passed — 2026-11-08

The verification API now has local-capture HTTP evidence, completing this backend verification task. Full auth/product acceptance remains open.
The combined auth HTTP workflow passes in 9.84 seconds. Target compilation, formatting, and whitespace checks pass.
No application edits, real emails, paid model calls, browser runs, full-suite rerun, or deployment occur.
Evidence: tests/artifacts/email-verification-http-2026-11-08/summary.md. All four PM docs reviewed; product scope stays unchanged.

## Next backend verification: email confirmation — 2026-11-08

Next: extend local email-capture HTTP coverage to verification requests and confirmation.
Cover signed-in request access, correct recipient/link, verified flag persistence, and rejection of invalid/expired/replayed tokens.
Existing validation and transaction tests do not cover this full HTTP flow. Browser pages remain deferred to frontend work.
Reviewed all four PM docs and existing auth HTTP coverage. No application edits or tests occur this turn. Scope and acceptance stay unchanged.

## Custom sessions replacement verified — 2026-11-08

Custom sessions replace Tower Sessions in the backend. Account-wide reset revocation now has HTTP and transaction rollback evidence. This completes the tested backend replacement, not the full recovery product story or auth epic.
All-target compilation passes. Auth database tests: 17 pass. Integration pure tests: 12 pass. Text types: eight pass.
Library tests: 13 pass and the known retrieval-ranking failure remains. HTTP recovery passes in dev and prod using loopback email capture.
Formatting and whitespace checks pass; QA fixture users/triggers are removed. No real emails, paid AI calls, browser tests, or deployment occur.
Evidence: tests/artifacts/custom-sessions-2026-11-08/summary.md. All four PM docs reviewed; scope stays unchanged.

## Logout cookie removal confirmed — 2026-11-08

Inspection confirms logout returns the updated CookieJar and removes session_id at Path=/ after deleting its database session.
Old Tower Sessions middleware still runs and uses the same cookie name. Remove its layer, helper/configuration, unused controller import, and dependencies before runtime verification.
Next verification: login/profile/logout plus two existing sessions invalidated by password reset.
Reviewed all four PM docs. No application edits, compilation, or tests occur; replacement acceptance remains pending.

## Custom logout review — 2026-11-08

Logout now resolves the custom session user and deletes the matching custom database session.
It still returns only JSON. The browser cookie is not removed; return CookieJar::remove with a session_id cookie using Path=/ alongside JSON.
The Tower Session import is now unused. Old middleware/configuration removal remains pending.
Reviewed all four PM docs. Inspection only: no application edits or tests. Replacement acceptance stays open.

## Custom UserId extractor review — 2026-11-08

Inspection confirms UserId now reads session_id through CookieJar and resolves identity through AuthService using AppState.
Missing cookies and missing/expired sessions return AuthError; database failures propagate separately.
Next: replace Tower-based logout with custom session deletion and matching cookie removal, then remove obsolete Tower middleware/configuration.
Reviewed all four PM docs. No application edits, compilation, or tests occur. Runtime acceptance stays pending.

## UserId cookie extraction guidance — 2026-11-08

Use CookieJar::from_headers(&parts.headers), then get("session_id") to read the raw token.
For the simplest state access, implement FromRequestParts<AppState> rather than generic S; the extractor can then call state.auth_service.get_user_id_from_session.
Missing cookies and absent/expired sessions map to AuthError. Propagate actual lookup errors through AppError.
Remove the old Tower Session import/extraction; the draft Cookie import also lacks a semicolon and should become CookieJar.
Guidance only: no application edits or tests. All four PM docs reviewed; extractor wiring remains pending.

## Login cookie settings implemented — 2026-11-08

Login cookie configuration is implemented and compilation is verified. The custom-session replacement remains partial; runtime recovery/session acceptance is not complete.
SQLX_OFFLINE=true cargo check --offline --jobs 2 --all-targets passes with existing warnings in 25.77 seconds.
Selected Rust files are formatted; git diff --check passes. No tests, email sends, model calls, or deployments occur.
All four PM docs reviewed. Product scope stays unchanged.

## Cookie builder guidance — 2026-11-08

Checked installed cookie 0.18.2 builder API. Use path, http_only, same_site, secure, expires, then build.
Convert the database chrono expiry timestamp to time::OffsetDateTime for expires. A direct time dependency is needed for that import.
Secure must use the configured environment; AppState currently does not expose an is_prod value, so that wiring remains necessary.
Guidance only: no dependency/application edits or tests. All four PM docs reviewed; integration remains pending.

## Custom login cookie review — 2026-11-08

Login now creates a custom database session and returns the updated CookieJar with the raw session token.
Cookie::new alone leaves auth-cookie policy unset. Set Path=/, HttpOnly, SameSite=Lax, Secure in production, and browser expiry matching the stored expiry.
Logout and UserId still use Tower Sessions. Login now uses the custom table, so authentication wiring is mixed and migration remains incomplete.
Reviewed all four PM docs. No application edits or tests occur this turn. Prior recovery results predate these changes and do not verify the new session path.

## Cookie extractor API guidance — 2026-11-08

Checked official axum-extra CookieJar documentation. Use its cookie feature and a CookieJar handler argument to read named cookies.
jar.get("session_id").map(|cookie| cookie.value()) gives the optional raw session token. A missing cookie means unauthenticated.
CookieJar add/remove changes reach the client only when the updated jar is returned in the response.
axum-extra is not yet a direct dependency. Guidance only: no dependency or application edits and no tests occur.
Reviewed all four PM docs. Custom-session cookie/extractor integration and acceptance remain pending.

## Session identity lookup review — 2026-11-08

Inspection confirms get_user_id_from_session returns Option<Uuid> after hashing the supplied token and querying an unexpired session.
Next: login sets the raw token cookie; UserId reads it and resolves the user through AuthService. Cookie policy and logout follow in the replacement.
Reviewed all four PM docs. No application edits or tests occur. Tower Sessions remains active; replacement and acceptance stay pending.

## Auth session service review — 2026-11-08

Inspection confirms create_session generates a raw token, persists its SHA-256 hash, and returns the token plus database expiry.
Session reads/deletes hash supplied tokens before repository calls. No raw-token logging appears in the new methods.
Remaining API issue: session_valid returns bool and discards the row's user_id, which the UserId extractor needs.
Recommend resolve_session returning Result<Option<Uuid>, ServiceError>, mapping an active row to its user_id.
Reviewed all four PM docs. No application edits or tests occur; cookie/extractor wiring and runtime replacement remain pending.

## Parent session methods review — 2026-11-08

UserRepository now exposes get_session, create_session, and delete_session delegating to its private child.
Inspection only; no compilation or tests. Next work is AuthService token generation/hashing and cookie/extractor wiring.
All four PM docs reviewed. Tower Sessions replacement and product acceptance remain pending; no application edits this turn.

## Session child and reset transaction review — 2026-11-08

Inspection confirms UserRepository constructs and holds SessionRepository; the child derives Debug.
Reset now consumes its token, updates the password, and deletes the user's custom sessions on the same transaction connection.
The existing error branch rolls back all three operations. Zero deleted sessions is allowed; it does not fail reset.
Tower Sessions still supplies active login/extractor behavior, so deleting custom rows does not yet revoke current Tower sessions.
Next: parent session create/read/delete methods, followed by AuthService, cookie, and UserId wiring to the custom table.
Reviewed all four PM docs. No application edits, compilation, or tests occur this turn. Runtime replacement and acceptance stay pending.

## Session lookup follow-up — 2026-11-08

Inspection confirms get now takes only token_hash and filters expired sessions. SessionRow supplies user_id.
Next: UserRepository must hold and construct the sessions child. Add Debug to the child to satisfy the parent's existing derive.
No application edits or tests occur this turn. All four PM docs reviewed; replacement and acceptance remain pending.

## Session repository review — 2026-11-08

Reviewed create/get/delete/delete_all queries and corrected migration syntax by inspection.
Expiry filtering exists; delete_all accepts the parent's connection for shared transactions.
Main API question: get requires user_id although the cookie carries only the random session token. Token-hash lookup can return user_id from SessionRow.
UserRepository declares the sessions module but does not yet construct or hold SessionRepository. Tower Sessions remains active.
Reviewed all four PM docs. No application edits, compilation, migrations, or tests occur this turn. Session replacement and acceptance remain pending.

## Sessions migration review — 2026-11-08

Reviewed migrations/20261108094008_sessions_table.sql. Columns match the proposed auth-session design.
The user_id foreign key cascades on deletion, and its index supports user-wide session deletion.
Syntax blocker: CREATE TABLE lacks its terminating semicolon before CREATE INDEX.
The expiry default sets a fixed 24-hour lifetime at insertion; it does not implement sliding expiry.
Reviewed all four PM docs. No migration edits, database application, or tests occur. Replacement and acceptance remain pending.

## Application-owned auth sessions proposal — 2026-11-08

User proposes replacing Tower Sessions with an application-owned session layer. Current use is login identity, logout, and expiry.
A focused design uses sessions(token_hash, user_id, expires_at), a random secret cookie, and a database-backed UserId extractor.
Login inserts a fresh session; logout deletes that session; reset deletes all user sessions within the existing password/token transaction.
Use established randomness, hashing, and cookie libraries. Replacing the session framework does not mean implementing cryptography or cookie parsing.
This moves responsibility for cookie policy, expiry, and concurrent login/reset behavior into the app. Tests must cover these before replacement is accepted.
Proposal only: no dependency removal, schema changes, application edits, or tests occur. Existing clients would need to log in again after a cutover.
Reviewed all four PM docs. Scope and epic acceptance remain unchanged.

## Direct session ownership alternative — 2026-11-08

User rejects session-version invalidation as an unwanted design. Do not implement it without a new decision.
Alternative: a user_id column owned and maintained by the session store, with an index for account-wide deletion.
A column alone is insufficient: the installed store never writes it. A custom SessionStore or store adapter must maintain ownership when creating/saving sessions.
Password reset can delete the user's session rows in its transaction. Concurrency behavior, including in-flight session saves, still needs design and tests.
This is an alternative for discussion, not an accepted implementation. More integration code is required than version checks.
Reviewed all four PM docs. No application changes or tests occur. Acceptance remains open.

## Session invalidation simplicity review — 2026-11-08

Checked installed PostgresStore methods: deletion takes a session ID; there is no user-wide deletion method.
Recommend the previously proposed per-user session version as the smallest focused implementation; do not scan binary session records.
This remains a recommendation, not approval to implement. No application changes or tests occur this turn.
Reviewed all four PM docs. Session invalidation and product acceptance remain open.

## Finding sessions by user — 2026-11-08

Installed PostgresStore serializes the full session Record with rmp_serde (MessagePack) into data BYTEA.
To find existing sessions for a user without schema changes: read unexpired id/data rows, decode each Record in Rust, and compare record.data["user_id"].
This requires scanning and decoding active sessions; the current table has no searchable user_id index.
This is an inspection approach, not an accepted or implemented account-wide revocation solution.
Reviewed all four PM docs. No application changes, session deletions, or tests occur. Acceptance stays open.

## Session store columns — 2026-11-08

Installed store schema has id TEXT primary key, data BYTEA, and expiry_date TIMESTAMPTZ; all are NOT NULL.
There is no separate user_id column. Session invalidation design remains open.
Reviewed all four PM docs. Source inspection only; no application changes or tests occur. Acceptance stays unchanged.

## Session store table lookup — 2026-11-08

Verified installed tower-sessions-sqlx-store-chrono 0.14.2 defaults: schema tower_sessions, table session.
Current session setup uses these defaults. User rejects the proposed version design as too complex; it remains unimplemented.
Next investigation follows the existing store. No replacement design is accepted yet.
Reviewed all four PM docs. No application changes or tests occur. Recovery acceptance stays open.

## Proposed session version design — 2026-11-08

Proposal: add users.session_version with default zero. Store the current version with user_id at login.
UserId loads the user's current version and rejects missing, mismatched, or deleted-user sessions with 401.
Password reset increments the version in the same transaction as the password update and token consumption.
This rejects all older sessions without scanning the session store. Existing sessions without a version must log in again.
It adds one user lookup to authenticated requests. Old stored session rows expire normally; rejection does not require bulk deletion.
Reviewed all four PM docs. Design only: no migration, application edits, or tests this turn. Acceptance remains open.

## Reset-handler session flush review — 2026-11-08

The handler now flushes the session attached to the reset request after a successful password change.
This clears only that requesting session. It does not revoke other browser/device sessions for the target user.
A logged-out reset request has no authenticated session to revoke. Account-wide session invalidation remains open.
Reviewed all four PM docs and the changed handler. No application edits or tests occur this turn. Product scope and acceptance stay unchanged.

## Next backend task: reset session invalidation — 2026-11-08

Prioritize password-reset session invalidation before more auth features. Keep browser confirmation pages deferred to frontend work. No implementation or acceptance changes this turn.
Reviewed all four PM docs and session/reset code. No application changes or tests occur this turn. Product scope stays unchanged.

## Recovery HTTP verification — 2026-11-08

Recovery route verification is complete for the tested API flow. Product recovery is not accepted until remaining flows and policies are handled.
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
Evidence: tests/artifacts/auth-recovery-http-2026-11-08/summary.md. Reviewed all four PM docs.

## Recovery verification priority — 2026-11-08

Prioritize HTTP recovery verification before adding auth features. Existing model and database tests do not verify controller wiring. No delivery status changes this turn.
Reviewed all four PM docs. No application changes or tests occur this turn. Product scope stays unchanged.

## Auth cleanup test run — 2026-11-08

Local library tests: 13 pass, one known retrieval-ranking failure, 16 database tests ignored by default.
Ran those 16 auth database tests separately against isolated QA Postgres: all pass in 6.90 seconds.
Explicit integration targets: 12 pass with 47 ignored; text-type tests: eight pass.
Total executed tests: 49 pass and one known failure. The full suite is not green.
The failure remains duplicate_rows_in_one_search_must_not_outvote_two_independent_searches in unchanged retrieval code.
Evidence: tests/artifacts/auth-cleanup-retest-2026-11-08/. No application changes, emails, paid model calls, browser runs, or deployments occur.
Reviewed all four PM docs. HTTP recovery coverage and product acceptance remain pending; scope stays unchanged.

## Reset API cleanup — 2026-11-08

Removed stale ResetPasswordRequest and Query imports, and the missing user_id tracing field.
The reset-request controller returns the agreed neutral success message. Unknown-account no-op behavior stays unchanged.
Adapted existing auth validation tests to separate PasswordResetParams and PasswordResetPayload. Added invalid-email validation coverage.
All five targeted auth request tests pass. The integration test target compiles with existing warnings.
Updated root README for public email requests and query/body confirmation inputs. Selected-file formatting and git diff --check pass.
No email sends, model calls, database regressions, or HTTP recovery tests run this turn.
Next: verify logged-out request and confirmation behavior through HTTP. Confirmation pages and session policy remain pending.
Reviewed all four PM docs. Product scope and epic acceptance stay unchanged.

## Missing-account branch review — 2026-11-08

Inspection confirms unknown emails now return Ok(()) before token creation or email delivery.
The neutral response text remains proposed: the controller still says "Password reset email sent."
Compile blockers remain: stale ResetPasswordRequest imports in the controller and auth request tests, and missing user_id in the sender tracing attribute.
Reviewed all four PM docs. No application changes or tests occur this turn. Recovery and epic acceptance remain incomplete.

## Unknown-email reset behavior — 2026-11-08

Decision: an unknown email skips token creation and email delivery, then returns Ok(()).
The controller uses the same neutral success message for known and unknown emails: "If an account exists, we will send a reset link."
Database and email-provider failures remain real errors; do not silently swallow them. This decision covers the missing-account branch only.
Reviewed all four PM docs. No application changes or tests occur this turn. Recovery and epic acceptance remain incomplete.

## Logged-out reset implementation review — 2026-11-08

By inspection, reset requests now accept validated email JSON without UserId. Confirmation separates query credentials from the password body.
Two compile blockers remain: the controller imports removed ResetPasswordRequest, and the email sender tracing attribute references missing user_id.
Unknown emails still return NotFound, so the public response reveals account existence. Recovery remains incomplete.
The changed confirmation contract also needs matching test and client updates before acceptance.
Reviewed all four PM docs. No application edits, compilation, tests, email sends, or model calls occur this turn.
Product scope and epic acceptance remain unchanged.

## Backend next-step review — 2026-11-08

Prioritize logged-out password recovery after the completed repository ownership refactor. Confirmation pages and session policy remain follow-up work. No delivery or epic acceptance changes this turn.
Reviewed all four PM docs and the auth controller. No application code changes or tests occur this turn.

## User repository owns auth transactions — 2026-11-08

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

## User repository transaction ownership proposal — 2026-11-08

Plan only: UserRepository owns its PgPool and privately constructs AuthTokenRepository from a cloned pool handle.
Proposed layout: repositories/user/mod.rs for the parent and repositories/user/auth_tokens.rs for token queries.
The parent exposes token issuance, reset_password_with_token, and verify_email_with_token.
Reset and verification methods begin one transaction, consume the matching valid token through the child, update the user, then commit.
A failed consume or user update rolls back; a missing user also rolls back before returning None. Preserve invalid-token and missing-user error distinctions.
Existing user-update queries become private connection-aware helpers. The child receives that same PgConnection for consumption and never commits it.
AuthService keeps password/submitted-token hashing, response mapping, URL building, and email delivery; its pool and direct token-repository dependency are removed.
Bootstrap constructs only UserRepository for the auth data dependency. Email/provider work and password hashing remain outside database transactions.
Adapt token lifecycle fixtures and retain rollback, replay, expiry, purpose, and concurrency coverage. No migration is proposed.
No application code changes or tests occur in this planning turn. Product scope, priorities, and epic acceptance remain unchanged.

## Analyzer cold-start source duplication — 2026-11-08

The analyzer remains CPU-heavy with Cargo idle; one fresh instance reaches about 4.3 GB resident memory after two minutes.
Confirmed main.rs and the integration harness independently declare the full application source tree.
Excluding tests does not reduce observed memory use and is reverted. Disabling native diagnostics also leaves heavy startup work and is reverted.
Added src/lib.rs. The binary and native integration harness now import that shared library rather than declaring application modules twice.
Two existing researcher entry points used by integration tests are public across the new crate boundary; their bodies are unchanged.
Test IDE services and native diagnostics remain enabled. No test assertions or application business logic change.
Updated library-test commands and project layout documentation.
Removed unused direct anyhow/clap dependencies; the full Cargo metadata package list drops from 823 to 810.
Final all-target test compilation passes with warnings in 54.95 seconds.
Library tests: 13 pass and the existing ranking failure remains. Integration pure tests: 11 pass; text-type tests: eight pass.
All 59 free database regressions pass in 19.19 seconds. No emails or paid AI calls occur.
Bounded profiling identifies costly VFS handling and repeated workspace fetches, but establishes no editor-startup speedup.
Cold start remains open. The three settings experiments show no verified improvement and are reverted.
Temporary RA_PROFILE/RA_LOG settings capture the next real editor startup. Full window reload is required: server restarts reuse old environment settings.
Next: inspect the real startup trace and remove temporary profiling settings after diagnosis.
Evidence: [cold-start investigation](../tests/artifacts/analyzer-cold-start-2026-11-08/summary.md).
Editor startup improvement remains unverified. Scope, priorities, remaining product work, and epic acceptance stay unchanged.

## Development compile and analyzer startup tuning — 2026-11-08

Found rust-analyzer still forces CARGO_INCREMENTAL=0 and Cargo uses full default debug data.
Enabled incremental reuse in the development profile and rust-analyzer's Cargo environment.
Development debug data now uses line-tables-only; source-line backtraces remain, full debugger local/type data does not.
Added a two-job Cargo default and changed the analyzer's Cargo invocation to two jobs.
Release settings and separate IDE/terminal caches remain unchanged. No caches are deleted.
Warmed the analyzer cache with online SQLx validation: profile-change preparation takes 231.29 seconds wall time.
The unchanged repeat check passes in 5.18 seconds, including a package-cache lock wait. Both checks pass with warnings.
This is not a changed-source rebuild or full IDE-startup benchmark; no before/after speedup is established.
The separate terminal build cache can still need its first dependency rebuild with the new profile.
Workspace JSONC and inherited Cargo arguments validate; whitespace checks pass.
Evidence: [development build tuning](../tests/artifacts/development-build-tuning-2026-11-08/summary.md).
README documents the tradeoff. No application logic, tests, email sends, or paid AI calls change or run.
Actual editor startup and editing latency remain unmeasured. Story scope, delivery priorities, and acceptance stay unchanged.

## Rust-analyzer responsiveness tuning — 2026-11-08

Measured the active analyzer at about 3.2 GB resident RAM and 1.3 GB process swap on a 12-thread host.
Language-server logs show garbage-collection pauses up to 2.28 seconds during file changes.
Updated workspace settings: four analysis workers, 64 cached syntax trees, and ignored instrument expansion from tracing-attributes.
Only the intentional attribute-expansion-disabled notice is suppressed; unlinked-file diagnostics remain enabled.
SQLx, Askama, other macros, check-on-save, and online database checks remain enabled.
Validated all rust-analyzer setting names against installed extension 0.3.3105 and parsed the workspace JSONC.
The extension log confirms receipt of four workers and the ignored instrument configuration.
Online SQLx API compilation passes with warnings in 1.06 seconds. Whitespace checks pass.
The existing process still has its old worker count and memory footprint; reload the VS Code window to fully apply startup settings.
Editing latency and memory improvements remain unmeasured. No application code, test run, email, or provider calls occur.
Delivery priorities, story scope, and epic acceptance remain unchanged.

## Module-tree IDE warning — 2026-11-08

Verified repositories/mod.rs declares graph, and graph/mod.rs declares entities.
The former graph.rs no longer exists; an open editor tab may still reference that deleted file.
The source module chain is intact. The warning is consistent with stale rust-analyzer state after the file move.
Next: close the deleted graph.rs tab and run rust-analyzer: Reload Workspace; restart the server if needed.
IDE recovery is not yet confirmed. No application code or settings changes and no tests occur.
Do not disable unlinked-file diagnostics. Delivery priorities, story scope, and acceptance remain unchanged.

## GraphRepository child repository refactor — 2026-11-08

Replaced graph.rs with graph/mod.rs and five private child repositories.
The constructor assembles children from clones of the same Graph handle.
Shared row and identifier helpers are private functions in helpers.rs.
The parent delegates the existing API and coordinates graph pages, batch writes, and conflict acceptance.
Children accept the parent's mutable transaction; only the parent commits or rolls back shared writes.
Source comparison confirms all 41 Cypher literals and 33 public signatures are preserved.
The first compile exposed an extracted transaction reborrow error; fixed before regression testing.
All-target offline compilation passes with warnings in 40.38 seconds. Formatting and whitespace checks pass.
Existing tests: 59 free database regressions pass in 27.02 seconds.
The application pure target has 13 passes; the integration pure target has 24 passes.
Both targets retain the same known duplicate-row ranking failure in unchanged in-memory ranking code.
The full suite is not green. That previously discussed issue remains outside this refactor.
No new tests, email sends, paid AI calls, browser run, Docker rebuild, or deployment occur.
Evidence: [refactor checks](../tests/artifacts/graph-repository-refactor-2026-11-08/summary.md).
Story scope, delivery priorities, and epic acceptance remain unchanged. Graph database regression checks pass.

## GraphRepository split proposal — 2026-11-08

Sketched child repositories for entities, relationships, conflicts, graph conversations, and graph maintenance.
The parent constructor assembles children using clones of the same neo4rs Graph handle.
The parent keeps the public API and coordinates get_graph, update_graph_batch, and accept_conflict.
Children own queries. Cross-repository writes receive the same mutable transaction from the parent.
Only the parent commits or rolls back cross-repository operations; no nested child transactions are proposed.
Entity search stays with entities. Shared row helpers remain private functions; no generic repository abstraction is proposed.
GraphConversationRepository remains distinct from the existing Postgres ConversationRepository.
This is a proposal, not an implemented refactor. No application code changes or tests occur.
Next: agree the split, then extract entity operations first while preserving existing signatures and behavior.
Delivery priorities, story requirements, and epic acceptance remain unchanged.

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
