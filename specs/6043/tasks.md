# Tasks: Issue #6043

## Ordered Tasks
- T1 (RED): Add failing websocket stale-timeout test and ping cadence expectation.
- T2 (GREEN): Implement heartbeat tick + stale timeout close behavior in stream loop.
- T3 (VERIFY): Run websocket module tests and service-api websocket route slices.
- T4 (REGRESSION): Run adjacent service-api endpoint tests plus fmt/clippy gate.

## Tier Mapping
- Unit: T1, T3
- Functional: T3
- Conformance: T1, T3
- Integration: T3
- Regression: T4
