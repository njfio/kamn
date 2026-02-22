# Tasks: #5776 Add Query-Task and Query-Agent-Profile Portable-Agent Surfaces

- [x] T1 (Conformance/RED): run affected CLI/MCP contract lanes and capture current failing/missing behavior.
- [x] T2 (Implementation): add CLI `query-task` and `query-agent-profile` command parse/dispatch/modules.
- [x] T3 (Implementation): add MCP `query_task` and `query_agent_profile` inventory and dispatch support.
- [x] T4 (Implementation): extend CLI/MCP contract tests (including real-backend integration for one new query route).
- [x] T5 (Implementation): perform compensating archived issue-spec pair cleanup and update `specs/archive/index.md`.
- [x] T6 (Conformance/GREEN): run targeted CLI/MCP lanes and archive policy checker.
- [x] T7 (Verify): run fmt/clippy and workspace gate.
- [x] T8 (Closure): set spec status Implemented, update milestone index, close issue.
