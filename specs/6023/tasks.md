# Tasks: Issue #6023

## Ordered Tasks
- T1 (RED): add direct M3 tests for exact-match ordering, invalid token/duplicate-id failures, and determinism stable/drift decisions.
- T2 (Implementation): add minimal reusable fixtures/helpers in the M3 test module.
- T3 (GREEN): run targeted `kamn-core` tests scoped to `data_layer_m3_blind_index_search`.
- T4 (Regression): verify deterministic error and reason-code assertions remain stable.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: T1, T3
- Conformance: T1, T3
- Regression: T4
