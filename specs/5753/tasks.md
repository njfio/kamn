# Tasks: #5753 Reconcile R52 Post-Publication Priority Summary Status Markers

- [x] T1 (Conformance/RED): add priority reconciliation assertions + consistency checks to `review_r52_branch_hygiene_reconciliation_docs_contract.rs` and run targeted test expecting failure.
- [x] T2 (Implementation): add marker-contract guidance for post-publication priority reconciliation in `docs/review/README.md`.
- [x] T3 (Implementation): add additive priority reconciliation marker block in `docs/review/gaps-and-issues-r52.md` while preserving Section 8 baseline rows.
- [x] T4 (Implementation): perform compensating single archived issue-spec pair cleanup for issue `3872` and update `specs/archive/index.md`.
- [x] T5 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract` and `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- [x] T6 (Verify): run archive-policy checker and companion docs-contract regression suites.
- [x] T7 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T8 (Closure): mark spec Implemented, update milestone/index closure markers, and close issue.
