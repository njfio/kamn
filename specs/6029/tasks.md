# Tasks: Issue #6029

## Ordered Tasks
- T1 (RED): Add failing tests for unsupported transport and dispatch projection envelope mapping.
- T2 (RED): Add failing tests for presence projection visible/not-found envelopes and audit tagging.
- T3 (GREEN): Apply minimal implementation adjustments only if failing tests expose a contract mismatch.
- T4 (VERIFY): Run targeted `kamn-core` M9 gateway bridge tests and confirm C-01..C-03 pass.
- T5 (REGRESSION): Run adjacent M9 realtime delivery tests to guard against regressions.

## Tier Mapping
- Unit: T1, T2, T4
- Functional: T1, T4
- Conformance: T2, T4
- Regression: T5
