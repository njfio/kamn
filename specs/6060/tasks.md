# Tasks: Issue #6060

## Ordered Tasks
- T1 (RED): Add restart replay regression tests proving nonce reuse is currently accepted after guard re-initialization.
- T2 (GREEN): Implement persistent sender nonce floor state and monotonic nonce enforcement in replay guard.
- T3 (GREEN): Wire replay guard initialization to service API state-derived replay persistence path.
- T4 (VERIFY): Run `cargo fmt --check`, `cargo clippy -p kamn-node -- -D warnings`, and targeted replay/auth tests.

## Tier Mapping
- Unit: T2, T3
- Functional: T1, T2
- Conformance: T1, T4
- Integration: T1, T4
- Regression: T4
- Property: N/A (deterministic map/order logic; covered by explicit edge tests)
- Contract/DbC: N/A (no DbC macros in module)
- Snapshot: N/A (no snapshot outputs)
- Fuzz: N/A (no parser/wire-format changes)
- Mutation: N/A (covered by workspace mutation gate in CI)
- Performance: N/A (auth correctness fix; no throughput target change)
