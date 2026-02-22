# Tasks: #5629 EVIDENCE Phase Activation

- T1 (RED): add failing command-contract tests for EVIDENCE phase PASS/FAIL semantics and lifecycle totals propagation.
- T2 (Implementation): wire EVIDENCE phase step/status/details to evidence_contract status context.
- T3 (Regression): verify runtime/live/evidence marker contracts remain stable.
- T4 (Docs): add R54 evidence-phase activation artifact + docs contract test; update milestone index progress references.
- T5 (Verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, and `cargo test -p kamn-e2e-harness`.
