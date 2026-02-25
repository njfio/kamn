# Tasks: Issue #5990

## Ordered Tasks
- T1 (RED): Add/adjust integration tests that fail with current synthetic content lifecycle route behavior.
- T2 (Implementation): Add durable content lifecycle persistence methods to `ServiceApiMessageStore`.
- T3 (Implementation): Wire content lifecycle routes in middleware to message-store methods with deterministic fail-closed errors.
- T4 (GREEN): Run targeted content lifecycle restart integration tests.
- T5 (Regression): Run cross-node relay durability regression test to verify baseline relay flow remains green.

## Tier Mapping
- Unit: T2
- Functional: T1, T3, T4
- Integration: T1, T4
- Regression: T5
- Conformance: T4, T5
