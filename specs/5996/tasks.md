# Tasks: Issue #5996

## Ordered Tasks
- T1 (RED): Add restart integration test for channel create durability with explicit state-file assertion.
- T2 (Implementation): Add durable `create_channel` method to `ServiceApiMessageStore`.
- T3 (Implementation): Wire middleware channel create route to message-store persistence.
- T4 (GREEN): Run targeted channel durability integration test and service-api module tests.
- T5 (Regression): Run relay durability integration test.

## Tier Mapping
- Unit: T2
- Functional: T1, T3, T4
- Integration: T1, T4
- Regression: T5
- Conformance: T4, T5
