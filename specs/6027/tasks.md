# Tasks: Issue #6027

## Ordered Tasks
- T1 (RED): Add failing tests for deterministic append/integrity and recall drift decision contracts.
- T2 (RED): Add failing tests for privacy-mode and tampered-chain fail-closed paths.
- T3 (GREEN): Adjust minimal implementation only if required to satisfy spec-derived failing tests.
- T4 (VERIFY): Run targeted `kamn-core` M5 tests and confirm C-01..C-04 pass.
- T5 (REGRESSION): Run adjacent data-layer module tests to guard against regressions.

## Tier Mapping
- Unit: T1, T2, T4
- Functional: T1, T4
- Conformance: T1, T4
- Regression: T2, T5
