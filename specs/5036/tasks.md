# Issue #5036 Tasks

- Issue: #5036
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing reconciliation conformance tests for match/mismatch, owner-scope denial, and invalid bucket alignment.
- [x] T2 (Green): implement additive M7 daily billing reconciliation API and typed invalid-bucket failure.
- [x] T3 (Refactor): preserve deterministic aggregate/projection outputs and stable reason markers.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, `cargo test -p kamn-core --test data_layer_m7_timeseries_telemetry`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): run shell guardrail evidence commands and confirm `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): set spec/plan/tasks status to `Implemented`/`Done`, map ACs to tests, and post closure evidence.
