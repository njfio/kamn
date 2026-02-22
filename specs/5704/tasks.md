# Tasks: #5704 Align `kamn-cli` Default/Output Contract with PRD JSON Semantics

- [x] T1 (Conformance/RED): add failing tests for JSON-default parser behavior and structured JSON renderer contract.
- [x] T2 (Implementation): introduce typed command output projections and wire command modules + main renderer.
- [x] T3 (Regression): run `cargo test -p kamn-cli`.
- [x] T4 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-cli -- -D warnings`.
- [x] T5 (Mutation): run `cargo mutants --in-diff` for `kamn-cli` slice.
- [x] T6 (Closure): set spec status to Implemented and close issue with telemetry.
