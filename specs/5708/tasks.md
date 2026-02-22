# Tasks: #5708 Reconcile R52 Branch Hygiene Drift with Merged-Only Cleanup and Docs-Contract Markers

- [x] T1 (Conformance/RED): add failing R52 branch-hygiene docs-contract test asserting reconciliation markers.
- [x] T2 (Implementation): capture pre-cleanup branch inventory and merged-only candidate list.
- [x] T3 (Implementation): execute merged-only remote branch cleanup wave and capture post-cleanup inventory.
- [x] T4 (Implementation): update `docs/review/gaps-and-issues-r52.md` branch-hygiene section with post-publication reconciliation markers.
- [x] T5 (Regression): run `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- [x] T6 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T7 (Closure): mark `specs/5708/spec.md` as `Implemented` and close issue markers.
