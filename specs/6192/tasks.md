# Tasks: Issue 6192 - MCP Tool Schemas Must Be Discoverable

- Issue: #6192
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): update schema contract tests to fail on wildcard schemas and require representative field declarations.
- [x] T2 (GREEN): replace wildcard schema constants with per-tool concrete input schemas and structured output schema.
- [x] T3 (REGRESSION): run `kamn-mcp-server` tool inventory and stdio protocol tests.
- [x] T4 (VERIFY): run `cargo fmt --check`, scoped clippy, and scoped tests.
