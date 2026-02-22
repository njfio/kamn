# Issue #5610 Tasks - External Execution Preflight Executable Diagnostics

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for non-executable and executable preflight behavior.
2. T2 (impl/green): implement executability checks + deterministic diagnostics.
3. T3 (docs/green): add R52 docs artifact and milestone index updates.
4. T4 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, `cargo test -p kamn-agent-lib`, `cargo test -p kamn-mcp-server -p kamn-cli`.
5. T5 (closeout): set spec status Implemented and complete PR/issue lifecycle artifacts.

## Test Tier Mapping
- Functional: preflight diagnostics behavior.
- Conformance: C-01..C-11.
- Integration: full harness command-contract suite.
- Regression: cross-crate package suite reruns.
- Unit/Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for this deterministic preflight hardening slice.
