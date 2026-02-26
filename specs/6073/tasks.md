# Tasks: Issue #6073

## Ordered Tasks
- T1 (RED): Add tests for DID-key map selection and single-key fallback.
- T2 (GREEN): Implement runtime state/env parse for DID-key registry.
- T3 (GREEN): Wire sender-specific key selection into auth verification.
- T4 (VERIFY): Run `cargo fmt --check`, `cargo clippy -p kamn-node -- -D warnings`, and targeted auth tests.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: T3
- Conformance: T1, T4
- Regression: T4
- Integration: N/A (auth helper and config plumbing)
- Property: N/A (finite selection logic)
- Contract/DbC: N/A (no DbC macros)
- Snapshot: N/A (no snapshots)
- Fuzz: N/A (no parser/wire-format changes)
- Mutation: N/A (workspace mutation gate in CI)
- Performance: N/A (no perf budget change)
