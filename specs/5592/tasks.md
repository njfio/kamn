# Issue #5592 Tasks - PRD Phase-6a Spawn Command-Plan Contracts

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for `spawn_plan` keys and canonical/mode-coherent templates.
2. T2 (impl/green): implement deterministic `spawn_plan` object in run output.
3. T3 (docs/green): add phase-6a docs marker artifact and milestone index update.
4. T4 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, and `cargo test -p kamn-mcp-server -p kamn-cli`.
5. T5 (closeout): set spec status Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Functional: spawn_plan marker behavior.
- Conformance: C-01..C-12.
- Integration: existing orchestration contracts + spawn_plan composition.
- Regression: phase-1/2/4 suite reruns.
- Unit/Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic contract slice.
