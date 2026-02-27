# Tasks: Issue #6143

## Ordered Tasks
- T1 (RED/Conformance): Capture missing inter-tick probe hook evidence in `runtime_orchestration.rs`.
- T2 (Implementation): Add one-shot inter-tick lane probe helper and wire into full-supervisor daemon loop.
- T3 (Implementation): Increase internal full-supervisor lane request budgets for startup/inter-tick/shutdown probe sequence.
- T4 (Regression): Add tests for one-shot probe completion and fail-closed non-success probe behavior.
- T5 (Verification): Run scoped `kamn-node` tests and record RED/GREEN evidence.
- T6 (Closure): Publish AC-to-test mapping and closure evidence in issue/PR.

## Tier Mapping
- Unit: T4, T5
- Functional: T2, T5
- Integration: N/A (no cross-crate/network integration contract changes)
- Regression: T4, T5
- Conformance: T1, T2, T4, T5, T6
