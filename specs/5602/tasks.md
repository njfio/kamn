# Issue #5602 Tasks - PRD Phase-6 Runtime External Process Orchestration

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for runtime orchestration role markers.
2. T2 (impl/green): implement deterministic runtime orchestration role-marker composition.
3. T3 (docs/green): add runtime orchestration docs marker artifact + milestone update.
4. T4 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, and `cargo test -p kamn-mcp-server -p kamn-cli`.
5. T5 (closeout): set spec status Implemented and complete PR/issue lifecycle artifacts.

## Test Tier Mapping
- Functional: runtime orchestration marker behavior.
- Conformance: C-01..C-12.
- Integration: run output contract composition with phase-6 guard/runtime markers.
- Regression: phase-1/2/4/5/6 suite reruns.
- Unit/Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic contract slice.
