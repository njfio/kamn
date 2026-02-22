# Issue #5615 Tasks - External Preflight Requires Absolute Binary Paths

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for relative-path rejection and absolute-path pass behavior.
2. T2 (impl/green): implement absolute-path checks in preflight with deterministic diagnostics.
3. T3 (docs/green): add R52 absolute-path diagnostics docs artifact + milestone index updates.
4. T4 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, and `cargo test -p kamn-mcp-server -p kamn-cli`.
5. T5 (closeout): set spec status Implemented and complete PR/issue lifecycle artifacts.

## Test Tier Mapping
- Functional: absolute-path preflight diagnostics behavior.
- Conformance: C-01..C-11.
- Integration: full harness command-contract suite.
- Regression: cross-crate package suite reruns.
- Unit/Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic preflight hardening slice.
