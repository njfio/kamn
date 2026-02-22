# Tasks: #5652 Verification Anchor Finality Value Contract

- [ ] T1 (Conformance/Functional): add RED tests for non-`FINAL` finality rejection in `command_contract.rs`.
- [ ] T2 (Unit): add RED test for deterministic invalid-finality diagnostics in `verify.rs`.
- [ ] T3 (Implementation): enforce exact `FINAL` value for `_verification.kolme_anchor.finality`.
- [ ] T4 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and `cargo test -p kamn-e2e-harness`.
- [ ] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [ ] T6 (Docs): add R61 research markers + docs contract coverage; set `spec.md` status to Implemented.
