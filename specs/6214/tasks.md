# Tasks: Issue 6214 - Parse MCP Success Responses as Structured JSON

- Issue: #6214
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add regressions for nested `ok=true` payloads where root `ok=false`.
- [x] T2 (GREEN): add shared JSON bool field helper and `result` scope lookup support.
- [x] T3 (GREEN): replace MCP driver substring success checks with structured bool checks.
- [x] T4 (REGRESSION): run `cargo test -p kamn-mcp-server` and `cargo test -p kamn-e2e-harness`.
- [x] T5 (VERIFY): run `cargo fmt --check` and scoped clippy with `-D warnings`.

