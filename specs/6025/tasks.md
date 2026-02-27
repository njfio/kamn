# Tasks: Issue #6025

## Ordered Tasks
- T1 (RED): add direct M4 tests for transition/visibility happy path, invalid transition + tamper failures, and reconciliation decisions.
- T2 (Implementation): add minimal reusable test fixtures/helpers in the M4 test module.
- T3 (GREEN): run targeted `kamn-core` tests scoped to `data_layer_m4_escrow_integration`.
- T4 (Regression): verify deterministic typed errors and reason-code markers.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: T1, T3
- Conformance: T1, T3
- Regression: T4
