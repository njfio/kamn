# Tasks: #5774 Reconcile R50 Doc-Contract Non-Regression Cap After R53 Lane Addition

- [x] T1 (Conformance/RED): reproduce failing lane `review_r50_doc_contract_consolidation_docs_contract`.
- [x] T2 (Implementation): reconcile R50 non-regression markers and matching test assertions (`95 -> 96`).
- [x] T3 (Implementation): perform compensating archived issue-spec pair cleanup and update `specs/archive/index.md`.
- [x] T4 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_doc_contract_consolidation_docs_contract`.
- [x] T5 (Conformance/GREEN): run related lanes (`review_r53_docs_contract`, `review_r50_spec_volume_remediation_docs_contract`) and archive policy checker.
- [x] T6 (Verify): run `cargo fmt --all --check`, `cargo clippy -p kamn-core --tests -- -D warnings`, and workspace gate.
- [x] T7 (Closure): set spec status to Implemented, update milestone index, close issue.
