# Research timing audit

Measured 2026-10-30 from the retained 2026-10-29 smoke logs in /tmp.
Start: first researcher `invoke_agent` event with `Current conversation Turns: 1/12`.
End: matching `research completed for conversation` event.
Runs are sequential. This measures the researcher agent call, excluding the preceding chat and job setup.

| Run | Jobs | Median seconds | Mean seconds | Minimum seconds | Maximum seconds |
| --- | ---: | ---: | ---: | ---: | ---: |
| First 20-turn world | 20 | 23.3 | 25.1 | 3.4 | 59.6 |
| Second 20-turn world | 20 | 27.0 | 25.5 | 3.0 | 52.5 |
| New 10-turn world | 10 | 21.1 | 21.9 | 9.9 | 37.6 |
| Combined | 50 | 23.3 | 24.6 | 3.0 | 59.6 |

Logs: `lorekeeper-smoke-20-2026-10-29.log`, `lorekeeper-smoke-20-2026-10-29-run2.log`, `lorekeeper-smoke-10-2026-10-29.log`.
Existing smoke report seconds include chat plus research and must not be treated as research-only times.
These results do not measure queue delay or concurrent production load.
