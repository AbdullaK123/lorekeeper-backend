# Auth cleanup verification

## Auth cleanup test run — 2026-11-08

Local library tests: 13 pass, one known retrieval-ranking failure, 16 database tests ignored by default.
Ran those 16 auth database tests separately against isolated QA Postgres: all pass in 6.90 seconds.
Explicit integration targets: 12 pass with 47 ignored; text-type tests: eight pass.
Total executed tests: 49 pass and one known failure. The full suite is not green.
The failure remains duplicate_rows_in_one_search_must_not_outvote_two_independent_searches in unchanged retrieval code.
Evidence: tests/artifacts/auth-cleanup-retest-2026-11-08/. No application changes, emails, paid model calls, browser runs, or deployments occur.
Reviewed all four PM docs. HTTP recovery coverage and product acceptance remain pending; scope stays unchanged.

