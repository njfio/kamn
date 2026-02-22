# Tasks: #5692 MCP JSON-RPC Stdio Protocol in `kamn-mcp-server`

- [x] T1 (Conformance/RED): add failing protocol tests for framed
  `initialize`/`tools/list`/`tools/call`/error handling and line-mode
  compatibility.
- [x] T2 (Implementation): add framing + JSON-RPC request/response helpers.
- [x] T3 (Integration): wire binary execution to framed MCP path and preserve
  newline JSON path.
- [x] T4 (Regression): run `cargo test -p kamn-mcp-server`.
- [x] T5 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-mcp-server -- -D warnings`.
- [x] T6 (Closure): set spec status to Implemented, update issue status to done,
  and post AC/test telemetry.
