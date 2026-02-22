# Tasks: #5698 Activate Opt-In Live CLI-Scripted S-01 Driver Execution

- [x] T1 (Conformance/RED): add failing tests for CLI live toggle and S-01
  runner pass/fail mapping.
- [x] T2 (Implementation): implement configurable `CliScriptedDriver` with
  env-based live toggle and default `kamn-cli health` runner.
- [x] T3 (Integration): wire run-contract CLI mode driver creation to
  `CliScriptedDriver::from_env`.
- [x] T4 (Regression): run `cargo test -p kamn-e2e-harness`.
- [x] T5 (Verify): run `cargo fmt --all --check` and
  `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T6 (Closure): set spec status to Implemented and close issue with
  conformance + test telemetry.
