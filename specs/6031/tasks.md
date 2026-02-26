# Tasks: Issue #6031

## Ordered Tasks
- T1 (RED): Add failing tests for node/edge lifecycle determinism and portable export ordering.
- T2 (RED): Add failing tests for cross-owner edge denial and duplicate edge-id rejection.
- T3 (RED): Add failing trust-propagation ranking/limit tests with stable reason-code assertions.
- T4 (GREEN): Apply minimal implementation updates only if tests expose contract defects.
- T5 (VERIFY): Run targeted `kamn-core` M6 tests and confirm C-01..C-04 pass.
- T6 (REGRESSION): Run adjacent data-layer module tests to guard regressions.

## Tier Mapping
- Unit: T1, T2, T3, T5
- Functional: T1, T5
- Conformance: T3, T5
- Regression: T2, T6
