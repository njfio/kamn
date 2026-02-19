# Issue #5103 Tasks

- Issue: #5103
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing tests for M10->M8 shred-completeness projection and fail-closed missing-message behavior.
- [x] T2 (Green): implement additive M10 projection contracts + bridge API for compliance-derived shred completeness.
- [x] T3 (Refactor): preserve existing M10 archive/recovery behavior without semantic changes.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, `cargo test -p kamn-core --test data_layer_m10_partition_archival`, `cargo test -p kamn-core --test data_layer_m10_partition_recoverability`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): run shell guardrail evidence commands and confirm `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): set spec/plan/tasks implemented and close issue with DoD markers.
