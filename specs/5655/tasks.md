# Tasks: #5655 Verification Hash Format Contract

- [x] T1 (Conformance/Functional): add RED tests for invalid hash-format rejection in `command_contract.rs`.
- [x] T2 (Unit): add RED tests for deterministic hash-format diagnostics in `verify.rs`.
- [x] T3 (Implementation): enforce non-empty `sha256:` format for `_verification.evidence_hash` and `_verification.kolme_anchor.tx_hash`.
- [x] T4 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T6 (Docs): add R62 research markers + docs contract coverage; set `spec.md` status to Implemented.
