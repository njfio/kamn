# Issue #5291 Tasks

## Ordered Execution (TDD-first)
- [x] T1 (Red): add failing conformance tests for within-budget, deterministic exceeded branches, invalid-limit fail-closed paths, and orchestration+budget composition (`C-01`..`C-05`).
- [x] T2 (Green): implement Phase-6 budget input/report contracts and deterministic evaluator (`C-01`..`C-03`).
- [x] T3 (Regression): add fail-closed invalid-budget validation with stable reason marker (`C-04`).
- [x] T4 (Verify): run fmt, strict clippy, and targeted test suites (`C-06`).
- [x] T5 (Closeout): open PR with AC mapping, RED/GREEN evidence, and shell-surface markers.

## Tier Mapping
- Unit: within-budget + exceeded dimension mapping.
- Functional: composed orchestration report budget evaluation.
- Integration: phase6 orchestration + budget evaluator.
- Regression: invalid budget limits.

## Dependencies
- Parent story: `#5253`
- Prior tasks: `#5285`, `#5287`, `#5289`
