# Tasks: #5646 Chain Dump Hash Continuity Verification

- [ ] T1 (Conformance/Functional): add RED tests for missing chain-dump block hash markers and continuity mismatch in `command_contract.rs`.
- [ ] T2 (Implementation): enforce deterministic block hash continuity checks in verify path.
- [ ] T3 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and resolve drift.
- [ ] T4 (Verify): run `cargo fmt --all --check`, strict clippy, and `cargo test -p kamn-e2e-harness`.
- [ ] T5 (Docs): add R59 research markers + docs contract coverage; set `spec.md` status to Implemented.
