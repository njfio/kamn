# Issue #5560 Tasks - PRD Phase-2 kamn-mcp-server and kamn-cli Foundation Implementation

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for phase-2 required paths, tool inventory, and CLI subcommands.
2. T2 (impl/green): add `crates/kamn-mcp-server` scaffold (`main`, `config`, `tools`) and workspace wiring.
3. T3 (impl/green): add `crates/kamn-cli` scaffold (`main`, `commands/*`) and workspace wiring.
4. T4 (impl/green): implement deterministic parser/dispatch behavior and output format/env contracts.
5. T5 (docs/green): add phase-2 gap/status research artifact and update milestone index active issue set.
6. T6 (verify): run fmt/clippy/targeted tests and capture RED->GREEN evidence.
7. T7 (closeout): set spec to Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Unit: parser/config/tool registry validation tests
- Functional: CLI/MCP dispatch behavior tests
- Conformance: C-01..C-09 via phase-2 tests and quality gates
- Integration: CLI/MCP wrappers invoking `kamn-agent-lib`
- Regression: rerun phase-1 + phase-2 targeted tests
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A (documented in PR)
