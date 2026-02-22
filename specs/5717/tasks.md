# Tasks: #5717 Execute R52 Spec-Volume Remediation Tranche-2 (14-Dir Reduction)

- [x] T1 (Conformance/RED): extend `review_r50_spec_volume_remediation_docs_contract.rs` with tranche-2 marker assertions; run targeted test expecting failure.
- [x] T2 (Implementation): capture tranche-2 pre-count and delete 14 archived issue-spec pairs (pointer + payload).
- [x] T3 (Implementation): update `specs/archive/index.md` (`archived_issue_count` and removed rows).
- [x] T4 (Implementation): update review artifacts (`gaps-and-issues-r52.md`, `gaps-and-issues-r50.md`) with tranche-2 and refreshed non-regression markers.
- [x] T5 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- [x] T6 (Verify): run archive policy checker and R52 reconciliation docs-contract regression suite.
- [x] T7 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T8 (Closure): mark spec status Implemented, set milestone/index closure markers, and close issue.
