# Issue #5013 Tasks

- Issue: #5013
- Status: Done

## Ordered Tasks
- [x] T1 (Red): define M10 red conformance tests for partition naming/planning,
      archival candidate selection, archival index integrity, and illegal transition guards in child task `#5026`.
- [x] T2 (Green): implement M10 partition/archival contracts and exports in child task `#5026`.
- [x] T3 (Refactor): tighten deterministic reason-marker behavior and fail-closed taxonomy in child task `#5026`.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m10_partition_archival`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero for child delivery (`shell_loc_delta_actual = 0`).
- [x] T6 (Verify): set story lifecycle artifacts to `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story issue with linked child evidence.
