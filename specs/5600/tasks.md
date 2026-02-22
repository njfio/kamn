# Issue #5600 Tasks - PRD Phase-6 Runtime External Execution Integration

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for parser flag, runtime integration markers, and deterministic preflight failures.
2. T2 (impl/green): implement `external_execution` config field, parser flag support, guarded preflight checks, and runtime integration output object.
3. T3 (docs/green): add runtime integration docs marker artifact and milestone index update.
4. T4 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, and `cargo test -p kamn-mcp-server -p kamn-cli`.
5. T5 (closeout): set spec status Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Functional: guarded external execution path behavior.
- Conformance: C-01..C-10.
- Integration: run output contract composition with prior phase-6 contracts.
- Regression: phase-1/2/4/5/6 suite reruns.
- Unit/Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for guarded contract integration slice.
