# Tasks: #5750 Reconcile R52 Post-Publication Spec-Volume Guardrail Status Markers

- [x] T1 (Conformance/RED): add reconciliation marker assertions + consistency checks to docs-contract test and run targeted test expecting failure.
- [x] T2 (Implementation): add marker-contract guidance for post-publication guardrail reconciliation in `docs/review/README.md`.
- [x] T3 (Implementation): add additive post-publication guardrail reconciliation section + markers in `docs/review/gaps-and-issues-r52.md` while preserving baseline snapshot lines.
- [x] T4 (Implementation): perform compensating single archived issue-spec pair cleanup and update `specs/archive/index.md`.
- [x] T5 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- [x] T6 (Verify): run archive-policy checker and companion docs-contract regression suites.
- [x] T7 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T8 (Closure): mark spec Implemented, update milestone/index closure markers, and close issue.
