# Tasks: #5714 Execute R52 Spec-Volume Remediation Tranche-1 (14-Dir Reduction)

- [x] T1 (Conformance/RED): extend existing spec-volume docs-contract suite with tranche-1 marker requirements and run targeted test expecting failure.
- [x] T2 (Implementation): capture pre-tranche `specs/` top-level count evidence and delete selected 14 archive/pointer directory pairs.
- [x] T3 (Implementation): update `specs/archive/index.md` (`archived_issue_count` + remove 14 rows).
- [x] T4 (Implementation): update review marker docs (`README`, `gaps-and-issues-r50.md`, `gaps-and-issues-r52.md`) with refreshed ratchet + tranche markers.
- [x] T5 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- [x] T6 (Verify): run archive policy checker command and R52 reconciliation regression suite.
- [x] T7 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T8 (Closure): mark `specs/5714/spec.md` as `Implemented`, finalize milestone and issue closure markers.
