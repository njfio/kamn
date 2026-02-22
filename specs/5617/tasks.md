# Issue #5617 Tasks - Fix integration_config Flag Mapping in Run Output

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for integration_config flag mapping.
2. T2 (impl/green): correct flag mapping in run output serialization.
3. T3 (docs/green): add R52 docs artifact + milestone index updates.
4. T4 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, `cargo test -p kamn-mcp-server -p kamn-cli`.
5. T5 (closeout): set spec status Implemented and complete PR/issue lifecycle artifacts.

## Test Tier Mapping
- Functional: integration_config mapping behavior.
- Conformance: C-01..C-12.
- Integration: full harness command-contract suite.
- Regression: cross-crate package suite reruns.
- Unit/Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic mapping bugfix slice.
