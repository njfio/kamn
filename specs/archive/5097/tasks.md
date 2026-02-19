# Issue #5097 Tasks

- Issue: #5097
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing tests for telemetry-to-observability projection and owner-scoped evaluation behavior.
- [x] T2 (Green): implement M7 projection/evaluation contracts using `ObservabilityMonitor`.
- [x] T3 (Refactor): keep existing aggregate/billing behavior unchanged while integrating observability path.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, `cargo test -p kamn-core --test data_layer_m7_timeseries_telemetry`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): run shell guardrail evidence commands and confirm `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): set spec/plan/tasks implemented/done and post issue closure evidence.
