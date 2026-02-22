# Tasks: #5649 Chain Dump Genesis Anchor Verification

- [x] T1 (Conformance/Functional): add RED tests for genesis-anchor mismatch behavior in `command_contract.rs`.
- [x] T2 (Unit): add RED unit test for deterministic genesis-anchor mismatch diagnostic in `verify.rs`.
- [x] T3 (Implementation): enforce first-block `GENESIS` anchor in verify chain-dump validation.
- [x] T4 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T6 (Docs): add R60 research markers + docs contract coverage; set `spec.md` status to Implemented.
