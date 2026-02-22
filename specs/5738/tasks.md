# Tasks: #5738 Execute R52 Spec-Volume Remediation Tranche-9 (14-Dir Reduction)

- [ ] T1 (Conformance/RED): update tranche marker expectations in `review_r50_spec_volume_remediation_docs_contract.rs` and run targeted test expecting failure.
- [ ] T2 (Implementation): capture tranche-9 pre-count and delete 14 archived issue-spec pairs (pointer + payload).
- [ ] T3 (Implementation): update `specs/archive/index.md` (`archived_issue_count` + removed rows).
- [ ] T4 (Implementation): refresh review markers in `gaps-and-issues-r52.md` and `gaps-and-issues-r50.md`.
- [ ] T5 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- [ ] T6 (Verify): run archive policy checker and companion docs-contract regression suites.
- [ ] T7 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T8 (Closure): mark spec Implemented, update milestone/index closure markers, and close issue.
