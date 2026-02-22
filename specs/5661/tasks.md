# Tasks: #5661 Verification Captured-At Format Contract

- [x] T1 (Conformance/Functional): add RED tests for malformed captured-at rejection in `command_contract.rs`.
- [x] T2 (Unit): add RED tests for deterministic captured-at format diagnostics in `verify.rs`.
- [x] T3 (Implementation): enforce RFC3339 UTC-Z `_verification.captured_at` format.
- [x] T4 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T6 (Docs): add R64 research markers + docs contract coverage; set `spec.md` status to Implemented.
