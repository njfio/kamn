# Tasks: #5631 TEARDOWN Phase Activation

- [x] T1 (Conformance/Functional): add RED tests for teardown step inventory/status/detail and lifecycle totals in `command_contract.rs`.
- [x] T2 (Implementation): activate teardown step/status/detail semantics in harness run contract generation.
- [x] T3 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and fix any contract drift.
- [x] T4 (Verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Docs): add R54 teardown activation research marker and docs contract coverage.
