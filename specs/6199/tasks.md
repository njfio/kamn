# Tasks: Issue 6199 - MCP Framed Input Must Enforce Max Content-Length

- Issue: #6199
- Milestone: `R59 Swarm Gap Closure`

## Ordered Tasks

- [x] T1 (RED): add failing unit tests for content-length cap helper boundaries.
- [x] T2 (GREEN): implement max framed content-length constant and validator.
- [x] T3 (GREEN): invoke validator before framed payload allocation.
- [x] T4 (REGRESSION): run persistent stdio session contract test.
- [x] T5 (VERIFY): run `cargo fmt --check` and scoped clippy/tests.
