# Issue #5010 Tasks

- Issue: #5010
- Status: Done

## Ordered Tasks
- [x] T1 (Red): define M7 red conformance tests for ingest indexing, aggregate rollups,
      billing projections, and owner-scope deny paths in child task `#5023`.
- [x] T2 (Green): implement M7 telemetry contracts and exports in child task `#5023`.
- [x] T3 (Refactor): tighten deterministic aggregate/billing marker behavior in child task `#5023`.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m7_timeseries_telemetry`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero for child delivery (`shell_loc_delta_actual = 0`).
- [x] T6 (Verify): set story lifecycle artifacts to `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story issue with linked child evidence.
