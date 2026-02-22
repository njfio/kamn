# Tasks: #5772 Complete R53 Review Marker Contract Coverage

- [ ] T1 (Conformance/RED): add new `review_r53_docs_contract.rs` tests for required key families and invariants; run lane expecting failure.
- [ ] T2 (Implementation): add missing required marker blocks to `docs/review/gaps-and-issues-r53.md`.
- [ ] T3 (Implementation): perform compensating archived issue-spec pair cleanup and update `specs/archive/index.md`.
- [ ] T4 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r53_docs_contract`.
- [ ] T5 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract` and archive-policy checker.
- [ ] T6 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T7 (Closure): mark spec Implemented, update milestone index, close issue.
