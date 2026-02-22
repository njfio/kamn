# Issue #5558 Tasks - PRD Phase-1 kamn-agent-lib Foundation Implementation

## Ordered Tasks
1. T1 (tests/red): add failing phase-1 conformance tests for crate/module/API presence and auth/envelope/proof behavior contracts.
2. T2 (impl/green): add `crates/kamn-agent-lib` with module scaffolds and minimal typed implementations.
3. T3 (impl/green): implement `KamnAgentHandle` operations mapped to existing service primitives and explicit unsupported-operation errors where applicable.
4. T4 (docs/green): add phase-1 gap analysis research doc and PRD status markers.
5. T5 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-agent-lib -- -D warnings`, and targeted tests; capture RED->GREEN evidence.
6. T6 (closeout): set spec status to Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Unit: identity/auth/nonce/error module tests
- Functional: handle API behavior + envelope construction assertions
- Conformance: C-01..C-08 mapped to phase-1 test suite
- Integration: auth roundtrip + proof verification adapter tests
- Regression: targeted reruns for touched contracts
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A (documented in PR with follow-up tracking as needed)
