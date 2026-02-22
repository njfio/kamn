# Tasks: #5658 Verification Anchor Height Format Contract

- [x] T1 (Conformance/Functional): add RED tests for non-numeric block-height rejection in `command_contract.rs`.
- [x] T2 (Unit): add RED tests for deterministic block-height format diagnostics in `verify.rs`.
- [x] T3 (Implementation): enforce numeric `_verification.kolme_anchor.block_height` format.
- [x] T4 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T6 (Docs): add R63 research markers + docs contract coverage; set `spec.md` status to Implemented.
