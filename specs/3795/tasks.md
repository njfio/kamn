# Issue #3795 Tasks

- Issue: #3795
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing selector/fast-mode leakage assertions to live transport CI exclusion policy checks.
- [x] T2 (Red): add failing strategy docs contract assertions for transport-fault-matrix marker/taxonomy parity.
- [x] T3 (Green): implement selector/leakage assertion support and docs parity updates.
- [x] T4 (Regression): rerun exclusion/docs contract suites and shell guardrails.
- [ ] T5 (Verify): open mergeable PR and close issue with DoD markers.

## Tier Mapping
- Unit: docs marker assertions for transport-fault-matrix parity.
- Functional: exclusion policy script checks for selector and fast-mode leakage.
- Integration: combined exclusion + docs contract execution.
- Regression: fail-closed drift checks for policy markers and command-surface boundaries.
- Performance: N/A (no runtime hot-path change).
