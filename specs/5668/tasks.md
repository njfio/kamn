# Tasks: #5668 Activate MCP Verify-Proof Tool via Agent-Lib

- [x] T1 (Conformance/Functional): add RED verify-proof success and malformed-input tests (C-01, C-02).
- [x] T2 (Regression): assert unsupported task/escrow operations remain deterministic unsupported responses (C-03).
- [x] T3 (Implementation): add `verify_proof` backend trait method and dispatcher handling (AC-1, AC-2, AC-3).
- [x] T4 (Regression): run `cargo test -p kamn-mcp-server --test tool_dispatch_contract` and `cargo test -p kamn-mcp-server`.
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-mcp-server -- -D warnings`.
- [x] T6 (Closeout): set `spec.md` status to Implemented and post issue process log with RED/GREEN evidence.
