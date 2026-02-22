# Tasks: #5706 Resolve R50 Spec-Volume Non-Regression Cap Breach Blocking Workspace Gate

- [x] T1 (Conformance/RED): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract` and capture failure baseline.
- [x] T2 (Implementation): refresh R50 non-regression marker values in `docs/review/gaps-and-issues-r50.md`.
- [x] T3 (Implementation): update fixed marker expectations in `review_r50_spec_volume_remediation_docs_contract.rs`.
- [x] T4 (Regression): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- [x] T5 (Regression): run `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract`.
- [x] T6 (Verify): run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [x] T7 (Workspace Gate): run `cargo test --workspace --locked --all-features --no-fail-fast`.
- [x] T8 (Closure): mark `specs/5706/spec.md` as `Implemented` and update issue/milestone status markers.
