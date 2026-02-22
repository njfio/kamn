# Tasks: #5705 Restore R52 Green-Main Quality Gates for Review Markers and Pre-Merge Workspace Testing

- [x] T1 (Conformance/RED): run `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract` and capture failing marker parse baseline.
- [x] T2 (Conformance/RED): add failing workflow contract test requiring pre-merge workspace test gate in `ci-fast-gate`.
- [x] T3 (Implementation): remove R51 marker backticks and add pre-merge workspace gate job to `ci-fast-gate`.
- [x] T4 (Regression): run `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract` and `cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract`.
- [x] T5 (Regression): run `cargo test -p kamn-cli` and `cargo test -p kamn-mcp-server`.
- [x] T6 (Verify): run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [x] T7 (Closure): mark `specs/5705/spec.md` as `Implemented` and update issue/milestone status markers.
