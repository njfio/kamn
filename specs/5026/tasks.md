# Issue #5026 Tasks

- Issue: #5026
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add `spec_c01`..`spec_c05` tests for partition naming, archival eligibility, archival index, and re-attachment lifecycle guards.
- [x] T2 (Green): implement `data_layer_m10_partition_archival` contracts to satisfy the red suite.
- [x] T3 (Refactor): tighten deterministic month arithmetic and stable reason-marker taxonomy.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, `cargo test -p kamn-core --test data_layer_m10_partition_archival`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm zero shell/python/workflow/template delta and record `shell_loc_delta_actual=0`.
- [x] T6 (Verify): set spec lifecycle status to `Implemented`, map ACs to tests, and post issue closure evidence.
