# Tasks: Issue #6064

## Ordered Tasks
- T1 (RED): Add identity policy tests for allow/deny matrix and blocked deterministic identity behavior.
- T2 (GREEN): Implement deterministic identity policy helper + env override.
- T3 (GREEN): Add API security warning docs and wire policy check into `from_agent_name`.
- T4 (VERIFY): Run `cargo fmt --check`, `cargo clippy -p kamn-agent-lib -- -D warnings`, and targeted identity tests.

## Tier Mapping
- Unit: T1, T2
- Functional: T1, T3
- Conformance: T1, T4
- Integration: N/A (single-module policy hardening)
- Regression: T4
- Property: N/A (no randomized invariant needed)
- Contract/DbC: N/A (no DbC macros)
- Snapshot: N/A (no stable snapshot output changes)
- Fuzz: N/A (no parser/wire change)
- Mutation: N/A (workspace mutation gate in CI)
- Performance: N/A (no hot-path budget change)
