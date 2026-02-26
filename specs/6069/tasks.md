# Tasks: Issue #6069

## Ordered Tasks
- T1 (RED): Add TLS resolution tests for missing env on loopback/non-loopback addresses.
- T2 (GREEN): Implement bind-aware TLS mode missing-env behavior.
- T3 (VERIFY): Run `cargo fmt --check`, `cargo clippy -p kamn-node -- -D warnings`, and targeted service-api server tests.

## Tier Mapping
- Unit: T1, T2
- Functional: T2
- Conformance: T1, T3
- Regression: T3
- Integration: N/A (config resolution logic only)
- Property: N/A (finite branch logic)
- Contract/DbC: N/A (no DbC macros)
- Snapshot: N/A (no snapshots)
- Fuzz: N/A (no parser/wire change)
- Mutation: N/A (workspace mutation gate in CI)
- Performance: N/A (no perf budget change)
