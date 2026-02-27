# Tasks: Issue #6128

## Ordered Tasks
- T1 (RED/Conformance): Add or run failing test(s) that reproduce `S-05` gap from the spec ACs.
- T2 (Implementation): Apply minimal remediation to satisfy AC-1 without unrelated refactors.
- T3 (GREEN/Regression): Add and run regression coverage for AC-2 with deterministic assertions.
- T4 (Verification): Run scoped unit/functional/conformance commands and record evidence.
- T5 (Closure): Update issue/pr docs, map ACs to tests, and publish closure evidence.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T3, T4
- Integration: T4 (when cross-module behavior is affected)
- Regression: T1, T3, T4
- Conformance: T1, T4, T5
