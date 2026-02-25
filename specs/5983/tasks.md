# Tasks: Issue #5983

## Ordered Tasks
- T1 (Integration/Conformance): Add failing two-node delivery tests (send -> daemon relay -> recipient read) and restart durability checks.
- T2 (Implementation): Add relay routing config parsing and validation for daemon runtime.
- T3 (Implementation): Implement daemon relay forwarding path with retry-preserving spool semantics and deterministic error markers.
- T4 (Implementation): Add recipient relay-ingest persistence path and idempotent storage behavior.
- T5 (Regression): Add duplicate spool/repeated tick idempotency guard tests.
- T6 (Verification): Run scoped node runtime/service-api suites and map AC -> tests.

## Tier Mapping
- Unit: T2, T4
- Functional: T3, T4
- Integration: T1, T3
- Conformance: T1, T6
- Regression: T1, T5
