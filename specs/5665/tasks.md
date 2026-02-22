# Tasks: #5665 Activate MCP Server Execution and List-Messages Parity

- [x] T1 (Unit/Conformance): add RED SDK tests for list-messages route composition/decoding (C-01).
- [x] T2 (Functional/Regression): add RED agent-lib tests for `list_messages` replacing `UnsupportedOperation` (C-02).
- [x] T3 (Implementation): implement SDK `list_channel_messages` and wire agent-lib `list_messages` (AC-1, AC-2).
- [x] T4 (Unit/Conformance): add RED MCP server dispatch tests for supported and unsupported tools (C-03, C-04).
- [x] T5 (Implementation): implement MCP request parsing + dispatch + structured responses (AC-3, AC-4).
- [x] T6 (Regression): run targeted crates tests (`kamn-sdk`, `kamn-agent-lib`, `kamn-mcp-server`).
- [x] T7 (Verify): run `cargo fmt --all --check` and crate-scoped `cargo clippy -- -D warnings` on touched crates.
- [x] T8 (Docs/Closeout): update PRD coverage note and set `spec.md` status to Implemented once ACs are fully verified.
