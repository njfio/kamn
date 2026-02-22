# Tasks: #5649 Chain Dump Genesis Anchor Verification

- [ ] T1 (Conformance/Functional): add RED tests for genesis-anchor mismatch behavior in `command_contract.rs`.
- [ ] T2 (Unit): add RED unit test for deterministic genesis-anchor mismatch diagnostic in `verify.rs`.
- [ ] T3 (Implementation): enforce first-block `GENESIS` anchor in verify chain-dump validation.
- [ ] T4 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and `cargo test -p kamn-e2e-harness`.
- [ ] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [ ] T6 (Docs): add R60 research markers + docs contract coverage; set `spec.md` status to Implemented.
