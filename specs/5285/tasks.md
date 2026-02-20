# Issue #5285 Tasks

## Ordered Execution (TDD-first)
- [x] T1 (Red): add failing conformance tests for retention-to-archival eligibility and denied precondition paths (`C-01`..`C-04`).
- [x] T2 (Green): implement minimal M8->M10 projection glue with stable fail-closed reason markers (`C-01`..`C-04`).
- [x] T3 (Functional): add runtime-facing projection behavior test coverage (`C-05`).
- [x] T4 (Verify): run `cargo fmt --check`, strict `clippy`, and targeted tests for Phase-6 slice (`C-06`).
- [ ] T5 (Closeout): open PR with AC mapping, RED/GREEN evidence, and shell-surface markers.

## Tier Mapping
- Unit: eligibility projection helpers and reason-code mapping.
- Functional: runtime-facing archival gate projection behavior.
- Integration: M8 lifecycle input composed with M10 archival decision.
- Regression: denied-path reason stability.

## Dependencies
- Parent story: `#5253`
- Prior phase completion: `#5252` (tasks `#5279`, `#5281`, `#5283`)
