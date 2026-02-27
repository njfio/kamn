# Tasks: Issue #6144

## Ordered Tasks
- T1 (RED/Baseline): Capture pre-change marker/test-surface metrics and failing assertions for
  over-expanded governance contract surface.
- T2 (Implementation): Simplify heavyweight governance contract tests to essential fail-closed
  checks and remove self-referential reconciliation loops.
- T3 (GREEN/Regression): Add/keep focused regression checks for missing required core markers.
- T4 (Verification): Run scoped `kamn-core` governance contract tests and quality gates.
- T5 (Closure): Publish measurable reduction evidence and AC-to-test mapping in PR.

## Tier Mapping
- Unit: T3, T4, T5
- Functional: T2, T3, T4
- Conformance: T2, T3, T4, T5
- Regression: T1, T4, T5
- Integration: N/A (documentation/test-contract-only surface)
