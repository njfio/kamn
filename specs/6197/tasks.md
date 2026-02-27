# Tasks: Issue 6197 - MCP Server Must Consume `--key-file` Identity Material

- Issue: #6197
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing unit tests for key-file loading/validation helpers.
- [x] T2 (GREEN): load key-file content and build explicit identity for `KamnAgentHandle`.
- [x] T3 (GREEN): preserve persistent stdio session behavior.
- [x] T4 (REGRESSION): run `kamn-mcp-server` unit + persistent session contract tests.
- [x] T5 (VERIFY): run `cargo fmt --check` and scoped clippy/tests for touched crates.
