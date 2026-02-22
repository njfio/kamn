# Issue #5578 Tasks - PRD Phase-4h Live Runtime Binary Config Contracts

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for parser and run output integration config contracts.
2. T2 (impl/green): extend `RunCommandConfig` and run parser with new flags and MCP mode validation.
3. T3 (impl/green): add deterministic `integration_config` object to run output.
4. T4 (docs/green): add phase-4h docs marker artifact and milestone index update.
5. T5 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, and `cargo test -p kamn-mcp-server -p kamn-cli`.
6. T6 (closeout): set spec status Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Unit: parser requirement/error checks.
- Functional: run output integration config markers.
- Conformance: C-01..C-10.
- Integration: parse + mode validation + run-output composition.
- Regression: phase-1/2/4 suite reruns.
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic non-critical contract slice.
