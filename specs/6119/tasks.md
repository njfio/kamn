# Tasks: Issue #6119

## Ordered Tasks
- T1 (RED): Add failing CLI integration tests for `--help`, `-h`, and `help` behavior and output markers.
- T2 (GREEN): Implement deterministic help text renderer and help-request classifier.
- T3 (GREEN): Wire main entrypoint to short-circuit help path with exit `0`.
- T4 (VERIFY): Run targeted `kamn-cli` tests, fmt, and clippy.

## Tier Mapping
- Unit: T2
- Functional: T1, T3
- Conformance: T1, T4
- Regression: T1, T4
- Integration: T1 (binary execution contracts)
- Property: N/A (no invariant/randomized input surface)
- Contract/DbC: N/A (no contracts macros in crate)
- Snapshot: N/A (no snapshot fixtures)
- Fuzz: N/A (no new parser entrypoint)
- Mutation: N/A (workspace mutation gate handled in CI)
- Performance: N/A (no hot-path behavior change)
