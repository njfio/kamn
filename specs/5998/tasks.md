# Tasks: Issue #5998

## Ordered Tasks
- T1 (RED): Add restart integration test for agent-profile durability with state-file assertions.
- T2 (Implementation): Add durable agent-profile map + query/init method in `ServiceApiMessageStore`.
- T3 (Implementation): Wire live `GET /v1/agents/{agent_did}` route in middleware to message-store path.
- T4 (GREEN): Run targeted agent-profile restart test.
- T5 (Regression): Run relay durability integration test.

## Tier Mapping
- Unit: T2
- Functional: T1, T3, T4
- Integration: T1, T4
- Regression: T5
- Conformance: T4, T5
