# Tasks: #5634 EVIDENCE Step Inventory Parity

- [x] T1 (Conformance/Functional): add RED tests for PRD evidence step inventory + fail-path propagation in `command_contract.rs`.
- [x] T2 (Implementation): expand EVIDENCE step records/details in harness run contract generation.
- [x] T3 (Regression): run `cargo test -p kamn-e2e-harness --test command_contract` and resolve contract drift.
- [x] T4 (Verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, and `cargo test -p kamn-e2e-harness`.
- [x] T5 (Docs): add R55 evidence-step-inventory research marker and docs contract coverage.
