# Tasks: #5765 Add R52 Governance-Feature 70/30 Target Reconciliation Contract

- [x] T1 (Conformance/RED): add governance-feature target reconciliation assertions in `review_r52_branch_hygiene_reconciliation_docs_contract.rs` and run target lane expecting failure.
- [x] T2 (Implementation): add README contract guidance for post-publication governance-feature target reconciliation.
- [x] T3 (Implementation): add additive marker block to `docs/review/gaps-and-issues-r52.md` preserving historical snapshot narrative.
- [x] T4 (Implementation): perform compensating single archived issue-spec pair cleanup and update `specs/archive/index.md`.
- [x] T5 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- [x] T6 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract` and archive-policy checker.
- [x] T7 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T8 (Closure): set spec status to Implemented, update milestone index, close issue.
