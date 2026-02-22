# Tasks: #5693 Mutation Hardening for `kamn-mcp-server` Protocol Helpers

- [x] T1 (Conformance/RED): add failing helper-branch tests in `protocol.rs`
  covering escaped mutant classes.
- [x] T2 (Implementation): adjust helper behavior only if needed to satisfy
  branch assertions while preserving external behavior.
- [x] T3 (Regression): run `cargo test -p kamn-mcp-server`.
- [x] T4 (Mutation): rerun `cargo mutants --in-diff ... --package kamn-mcp-server`
  and capture caught/missed delta versus baseline.
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-mcp-server -- -D warnings`.
- [x] T6 (Closure): set spec status to Implemented and post telemetry to issue.
