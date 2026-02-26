# Tasks: Issue #6003

## Ordered Tasks
- T1 (RED): Add failing full-mode tests that simulate service-api and observability lane early exits during daemon execution and assert deterministic liveness failure reason codes.
- T2 (Implementation): Add runtime liveness monitor loop for full-supervisor lanes during daemon execution.
- T3 (Implementation): Reserve one full-mode lane request slot for startup probe + runtime lifecycle without changing CLI-level full-mode contract checks.
- T4 (GREEN): Run targeted full-supervisor runtime tests.
- T5 (Regression): Run `kamn-node` scoped tests + clippy/fmt to verify no full-mode behavior regressions.

## Tier Mapping
- Unit: T1, T2, T4
- Functional: T1, T2, T4
- Integration: T4
- Regression: T5
- Performance: N/A (control-flow checks only)
