# Tasks: Issue #5994

## Ordered Tasks
- T1 (RED): Add restart integration test for bridge lifecycle persistence and fail-closed missing bridge ID behavior.
- T2 (Implementation): Add durable bridge record schema + submit/forward/query methods to `ServiceApiMessageStore`.
- T3 (Implementation): Wire bridge routes in middleware to message-store methods with deterministic error mapping.
- T4 (GREEN): Run targeted bridge restart integration test and service-api module tests.
- T5 (Regression): Run durable relay integration test to confirm existing cross-node flow remains green.

## Tier Mapping
- Unit: T2
- Functional: T1, T3, T4
- Integration: T1, T4
- Regression: T5
- Conformance: T4, T5
