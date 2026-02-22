# Issue #5584 Tasks - PRD Phase-5a Process Runtime Inventory Contracts

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for process_runtime markers and mode-aware agent_runtime mapping.
2. T2 (impl/green): implement `process_runtime` run-output composition.
3. T3 (docs/green): add phase-5a docs marker artifact and milestone index update.
4. T4 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, and `cargo test -p kamn-mcp-server -p kamn-cli`.
5. T5 (closeout): set spec status Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Functional: process_runtime marker behavior.
- Conformance: C-01..C-11.
- Integration: mode + runtime inventory composition.
- Regression: phase-1/2/4 suite reruns.
- Unit/Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic contract slice.
