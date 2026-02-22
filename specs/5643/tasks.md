# Tasks: #5643 Chain Dump Marker Validation

- [x] T1 (Conformance/Functional): add RED tests for missing chain dump markers in `command_contract.rs`.
- [x] T2 (Implementation): add deterministic chain dump marker validation in verify flow.
- [x] T3 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and resolve drift.
- [x] T4 (Verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Docs): add R58 chain-dump hardening research marker and docs contract coverage.
