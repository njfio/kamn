# Tasks: Issue #5845

## Ordered Tasks
- [x] T1 (RED): add integration/regression tests for task/escrow persistence and recipient delivery status transition.
- [x] T2 (GREEN): extend `ServiceApiMessageStore` snapshot schema with task/escrow records and recipient mailbox projection.
- [x] T3: wire task/escrow/message recipient handlers in middleware to persisted store operations.
- [x] T4: preserve response compatibility while exposing optional delivery metadata on message query.
- [x] T5 (VERIFY): run scoped `kamn-node` route tests and full `kamn-node` test lane.

## Tier Mapping
- Unit: message/task/escrow store transition helpers.
- Functional: recipient mailbox projection and retrieval flow.
- Integration: task/escrow route lifecycle persistence through endpoint middleware.
- Regression: legacy message query flow stays `created` without recipient metadata.
