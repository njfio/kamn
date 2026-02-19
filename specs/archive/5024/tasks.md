# Issue #5024 Tasks

- Issue: #5024
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add `spec_c01`..`spec_c05` tests for CEK tombstoning, retention-class due windows, legal-hold precedence, and owner-scope fail-closed controls.
- [x] T2 (Green): implement `data_layer_m8_compliance_lifecycle` contracts to satisfy the red suite.
- [x] T3 (Refactor): tighten deterministic ordering and reason-code taxonomy for due-candidate outputs.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm zero shell/python/workflow/template delta and record `shell_loc_delta_actual=0`.
- [x] T6 (Verify): set spec lifecycle status to `Implemented`, map ACs to tests, and post issue closure evidence.
