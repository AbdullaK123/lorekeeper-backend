# GraphRepository refactor — 2026-11-08

The parent assembles five private child repositories. It retains the existing API and shared transaction ownership.
Source comparison preserves all 41 Cypher literals and 33 public signatures, including the constructor.
The initial compile found an extracted mutable transaction reborrow error; this is corrected.

All-target offline compilation passes with warnings in 40.38 seconds.
Formatting and whitespace checks pass.

## Existing test results

- Application pure target: 13 pass; one known ranking test fails.
- Integration pure target: 24 pass; the same ranking test fails; 61 live tests remain ignored.
- Free native regressions: all 59 pass in 27.02 seconds.
- Database regressions use isolated QA Postgres and fresh graph world IDs.
- Paid smoke and the HTTP wrapper are excluded. No email sends or paid AI calls occur.

The failing test is duplicate_rows_in_one_search_must_not_outvote_two_independent_searches.
It exercises RetrievalService::rrf using in-memory lists. It makes no repository calls.
Ranking source and its test remain unchanged. This previously discussed issue stays outside this refactor.
The full suite is therefore not green.

No Docker rebuild, container restart, browser run, or deployment occurs.
