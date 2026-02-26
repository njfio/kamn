# Tasks: Issue #6019

## Ordered Tasks
- T1 (RED): Add direct M1 tests for batch/proof happy path, invalid hash/proof failures, and failure-matrix decisions.
- T2 (Implementation): Add reusable fixture helpers inside `data_layer_m1` test module.
- T3 (GREEN): Run targeted `kamn-core` tests scoped to `data_layer_m1`.
- T4 (Regression): Confirm no unrelated behavior changes.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: T1, T3
- Conformance: T1, T3
- Regression: T4
