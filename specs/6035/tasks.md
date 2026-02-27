# Tasks: Issue #6035

## Ordered Tasks
- T1 (RED): Add failing tests for deterministic registry sequencing and due projection ordering.
- T2 (RED): Add failing tests for legal-hold fail-closed shred behavior and owner-scope boundary enforcement.
- T3 (GREEN): Apply minimal implementation changes only if tests expose a contract mismatch.
- T4 (VERIFY): Run targeted M8 tests and confirm C-01..C-03 pass.
- T5 (REGRESSION): Run adjacent M7/M9 slices to guard regressions.

## Tier Mapping
- Unit: T1, T2, T4
- Functional: T1, T4
- Conformance: T2, T4
- Regression: T5
