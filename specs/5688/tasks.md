# Tasks: #5688 Decompose `kamn-e2e-harness` Run-Contract Monolith

- [x] T1 (Baseline/Conformance): capture current `lib.rs` line count and targeted test baseline.
- [x] T2 (Implementation): extract run-contract logic from `lib.rs` into `run_contract.rs`.
- [x] T3 (Integration): rewire exports so public API surface remains unchanged.
- [x] T4 (Regression): run harness contract suites (`command_contract`, `mode_scenario_manifest_contract`).
- [x] T5 (Verify): run `cargo test -p kamn-e2e-harness` and `cargo fmt --all --check`.
- [x] T6 (Closure): set spec status to Implemented and close issue with LOC + test telemetry.
