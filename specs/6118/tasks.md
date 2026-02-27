# Tasks: Issue #6118

## Ordered Tasks
- T1 (RED): Add failing startup/integration tests proving `--key-file` must be readable and used.
- T2 (GREEN): Implement key-file identity loader and wire startup through `with_identity`.
- T3 (GREEN): Add unit tests for loader success/failure boundaries.
- T4 (VERIFY): Run targeted MCP tests, fmt, and clippy.

## Tier Mapping
- Unit: T3
- Functional: T2
- Conformance: T1, T4
- Integration: T1, T4
- Regression: T4 (persistent stdio session contract)
- Property: N/A (no randomized invariant changes)
- Contract/DbC: N/A (no contracts macros in touched modules)
- Snapshot: N/A (no snapshot fixture updates)
- Fuzz: N/A (no new parser entrypoint)
- Mutation: N/A (workspace mutation gate managed in CI)
- Performance: N/A (startup wiring only)
