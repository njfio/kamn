# Tasks: #5640 Evidence `_verification` Block Enforcement

- [ ] T1 (Conformance/Functional): add RED tests for missing `_verification` markers in `command_contract.rs`.
- [ ] T2 (Implementation): add deterministic evidence artifact marker validation in verify flow.
- [ ] T3 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and resolve drift.
- [ ] T4 (Verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, and `cargo test -p kamn-e2e-harness`.
- [ ] T5 (Docs): add R57 verification-block research marker and docs contract coverage.
