# Tasks: #5620 SCENARIO_RUN Driver Execution Contract

- T1 (Conformance/Functional, RED first): add failing tests for `scenario_results` output, order preservation, SCENARIO_RUN PASS/FAIL semantics, and lifecycle summary fail propagation.
- T2 (Implementation): add driver resolution + per-scenario execution helper and normalized status mapping in `src/lib.rs`.
- T3 (Implementation): update phase-step/status/details logic for `SCENARIO_RUN` based on execution outcomes.
- T4 (Regression): ensure existing runtime marker contracts remain unchanged and tests continue passing.
- T5 (Docs/Traceability): add R53 research artifact and docs contract test; update milestone index completed/active markers.
- T6 (Verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`.
