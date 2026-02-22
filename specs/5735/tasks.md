# Tasks: #5735 Execute R52 Spec-Volume Remediation Tranche-8 (14-Dir Reduction)

- [x] T1 (Conformance/RED): update tranche marker expectations in `review_r50_spec_volume_remediation_docs_contract.rs` and run targeted test expecting failure.
- [x] T2 (Implementation): capture tranche-8 pre-count and delete 14 archived issue-spec pairs (pointer + payload).
- [x] T3 (Implementation): update `specs/archive/index.md` (`archived_issue_count` + removed rows).
- [x] T4 (Implementation): refresh review markers in `gaps-and-issues-r52.md` and `gaps-and-issues-r50.md`.
- [x] T5 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- [x] T6 (Verify): run archive policy checker and companion docs-contract regression suites.
- [x] T7 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [x] T8 (Closure): mark spec Implemented, update milestone/index closure markers, and close issue.
