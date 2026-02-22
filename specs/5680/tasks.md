# Tasks: #5680 External Runtime Probe Execution in E2E Harness

- [x] T1 (Conformance/Functional): add RED tests for external probe-backed PASS/FAIL runtime marker behavior in `command_contract.rs`.
- [x] T2 (Unit): add runtime probe aggregation tests for status mapping and validation summary coherence.
- [x] T3 (Implementation): execute configured probe commands for external mode and wire outputs into runtime marker blocks.
- [x] T4 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T6 (Docs): update phase-6 runtime research docs and set spec status to Implemented.
