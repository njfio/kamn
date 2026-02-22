# Tasks: #5696 Activate Opt-In Live SDK-Direct S-01 Driver Execution

- [x] T1 (Conformance/RED): add failing unit tests for SDK-direct live toggle
  and S-01 probe pass/fail mapping.
- [x] T2 (Implementation): implement configurable `SdkDirectDriver` with live
  toggle and default `kamn-agent-lib` probe.
- [x] T3 (Integration): wire run-contract mode driver creation to SDK driver
  `from_env` constructor.
- [x] T4 (Regression): run `cargo test -p kamn-e2e-harness`.
- [x] T5 (Verify): run `cargo fmt --all --check` and
  `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T6 (Closure): set spec status to Implemented and close issue with
  conformance + test telemetry.
