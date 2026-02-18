# Issue #5041 Tasks

- Issue: #5041
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add `spec_c01`..`spec_c05` shell-neutral policy tests for verified/warn/blocked outcomes and fail-closed threshold validation.
- [x] T2 (Green): implement `data_layer_shell_neutral_policy` contracts and decision evaluator.
- [x] T3 (Refactor): tighten deterministic reason-marker ordering and threshold validation.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, `cargo test -p kamn-core --test data_layer_shell_neutral_policy`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm zero shell/python/workflow/template delta and record shell guardrail evidence.
- [x] T6 (Verify): set spec lifecycle status to `Implemented`, map ACs to tests, and post issue closure evidence.
