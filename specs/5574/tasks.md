# Issue #5574 Tasks - PRD Phase-4f Mode-Aware Lifecycle Population Contracts

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for mode-aware MCP step statuses and controlled fail-path markers.
2. T2 (impl/green): add deterministic mode-aware lifecycle status rule implementation.
3. T3 (impl/green): add controlled deterministic fail-path propagation to INFRA_UP step and phase status.
4. T4 (docs/green): add phase-4f docs marker artifact and milestone index state update.
5. T5 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, and phase-1/2 regressions.
6. T6 (closeout): set spec status Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Unit: mode-aware status rules.
- Functional: run output mode-aware/fail-path marker behavior.
- Conformance: C-01..C-10.
- Integration: parser + mode + phase/result/step composition.
- Regression: phase-1/2/4 suite reruns.
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic contract slice.
