# Tasks: Issue #6087

## Ordered Tasks
- T1 (RED): Add workflow policy contract assertions for panic-surface gate markers; run policy script and capture failure before workflow wiring.
- T2 (Implementation): Add panic-surface checker step + report artifact upload to fast-gate workflow.
- T3 (GREEN): Re-run workflow policy contract script and local checker command to verify non-regression.
- T4 (Regression): Ensure existing production-target `expect()` gate remains unchanged in scope (`--lib --bins`, no `--all-targets` widening).

## Tier Mapping
- Unit: N/A (workflow + shell contract only)
- Functional: T2, T3
- Integration: T2 (CI workflow wiring)
- Regression: T1, T4
- Conformance: T1, T3
