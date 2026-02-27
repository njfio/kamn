# Tasks: Issue #6033

## Ordered Tasks
- T1 (RED): Add failing tests for deterministic registration/planning and archival due projection ordering.
- T2 (RED): Add failing tests for invalid reattach transition and recovery-readiness projection behavior.
- T3 (GREEN): Apply minimal implementation changes only if failing tests expose a contract mismatch.
- T4 (VERIFY): Run targeted M10 registry tests and confirm C-01..C-04 pass.
- T5 (REGRESSION): Run adjacent M10 shared/retry test slices to guard regressions.

## Tier Mapping
- Unit: T1, T2, T4
- Functional: T1, T4
- Conformance: T2, T4
- Regression: T5
