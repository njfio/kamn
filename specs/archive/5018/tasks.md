# Issue #5018 Tasks

- Issue: #5018
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add `spec_c01`..`spec_c06` tests for DID session auth, ABAC matrix, RLS policy templates, and audit hash-chain tamper checks.
- [x] T2 (Green): implement `data_layer_m2_gateway_access` contracts to satisfy the red suite.
- [x] T3 (Refactor): tighten helper APIs and reason-code/error taxonomy clarity while preserving deterministic behavior.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, `cargo test -p kamn-core --test data_layer_m2_gateway_access`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm zero shell/python/workflow/template changes and record `shell_loc_delta_actual=0`.
- [x] T6 (Verify): set spec status to `Implemented`, complete AC mapping, and post issue process-log evidence.
