# Issue #5015 Tasks

- Issue: #5015
- Status: Done

## Ordered Tasks
- [x] T1 (Red): define failing conformance tests for required PRD critical-scenario catalog and fail-closed invalid/mutating paths in child task `#5028`.
- [x] T2 (Green): implement deterministic conformance evaluator + shell-neutral orchestration policy contracts in child task `#5028`.
- [x] T3 (Refactor): tighten reason-marker determinism and output ordering guarantees in child task `#5028`.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_prd_critical_scenario_conformance`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero for child delivery (`shell_loc_delta_actual = 0`).
- [x] T6 (Verify): set story lifecycle artifacts to `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story issue with linked child evidence.
