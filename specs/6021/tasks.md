# Tasks: Issue #6021

## Ordered Tasks
- T1 (RED): add direct M2 tests for audit append/verify, tamper detection, and matrix stable/drift decisions.
- T2 (Implementation): add minimal test fixtures/helpers inside `data_layer_m2_gateway_access` test module.
- T3 (GREEN): run targeted `kamn-core` tests scoped to `data_layer_m2_gateway_access`.
- T4 (Regression): confirm deterministic error/decision markers for tamper and drift paths.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: T1, T3
- Conformance: T1, T3
- Regression: T4
