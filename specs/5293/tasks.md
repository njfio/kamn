# Issue #5293 Tasks

- [ ] T1 (Red): add failing conformance tests for scheduler trigger decision precedence, deferred cycle no-op behavior, preflight budget overflow fail-closed path, and triggered cycle integration (`C-01`..`C-06`).
- [ ] T2 (Green): implement Phase-6 scheduler trigger policy/signal/decision contracts and evaluator with deterministic reason markers (`C-01`..`C-03`).
- [ ] T3 (Green): implement guarded scheduler-cycle contract with preflight budget admission + execution composition (`C-04`..`C-06`).
- [ ] T4 (Regression): ensure preflight overflow and invalid scheduler policy inputs fail closed deterministically (`C-05`).
- [ ] T5 (Verify): run `cargo fmt --check`, strict `clippy`, targeted M10 tests, and docs-contract tests (`C-07`).
- [ ] T6 (Closeout): open PR with AC mapping, RED/GREEN evidence, and shell-surface markers.
