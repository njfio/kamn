# Issue #5095 Tasks

- Issue: #5095
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing tests for typed conversion contracts and malformed
  DID fail-closed behavior.
- [x] T2 (Green): implement typed validated auth/scope contracts and boundary
  `TryFrom` conversions.
- [x] T3 (Refactor): route auth/ABAC internals through typed contracts while
  keeping public API signatures unchanged.
- [x] T4 (Regression): run `cargo fmt --check`,
  `cargo clippy -p kamn-core -- -D warnings`,
  `cargo test -p kamn-core --test data_layer_m2_gateway_access`, and
  `cargo test -p kamn-core`.
- [x] T5 (Governance): run shell guardrail evidence commands and confirm
  `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): set spec/plan/tasks implemented/done and post issue closure
  evidence.
