# Tasks: #5637 Verify Manifest Nested Field Hardening

- [x] T1 (Conformance/Functional): add RED tests for missing infrastructure/summary nested markers in `command_contract.rs`.
- [x] T2 (Implementation): expand `verify_manifest` nested-field checks with deterministic missing-field errors.
- [x] T3 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and resolve contract drift.
- [x] T4 (Verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Docs): add R56 verify-manifest hardening research marker and docs contract coverage.
