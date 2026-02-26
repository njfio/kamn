# Tasks: Issue #6066

## Ordered Tasks
- T1 (RED): Add clone-derive contract test for signer adapter.
- T2 (GREEN): Remove `Clone` derive from signer adapter.
- T3 (VERIFY): Run `cargo fmt --check`, `cargo clippy -p kamn-node -- -D warnings`, and targeted signer tests.

## Tier Mapping
- Unit: T2
- Functional: T3
- Conformance: T1, T3
- Regression: T1, T3
- Integration: N/A (single-type hardening)
- Property: N/A (no invariant generator needed)
- Contract/DbC: N/A (no DbC macros)
- Snapshot: N/A (no snapshots)
- Fuzz: N/A (no parser/untrusted input change)
- Mutation: N/A (workspace mutation gate in CI)
- Performance: N/A (no runtime behavior/perf delta)
