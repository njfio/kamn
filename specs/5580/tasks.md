# Issue #5580 Tasks - PRD Phase-4i CI Live-Lane Integration and Hardening Contracts

## Ordered Tasks
1. T1 (tests/red): add failing workflow/docs conformance tests for phase-4i markers.
2. T2 (impl/green): add `.github/workflows/e2e-live.yml` with PRD lane markers.
3. T3 (docs/green): add phase-4i docs marker artifact and milestone index update.
4. T4 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, and `cargo test -p kamn-mcp-server -p kamn-cli`.
5. T5 (closeout): set spec status Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Functional: workflow lane/mode marker checks.
- Conformance: C-01..C-12.
- Integration: workflow lane to harness mode coupling markers.
- Regression: phase-1/2/4 suite reruns.
- Unit/Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for workflow contract scaffold slice.
