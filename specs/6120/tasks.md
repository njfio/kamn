# Tasks: Issue #6120

## Ordered Tasks
- T1 (RED): Add failing integration test for oversized framed `Content-Length` rejection.
- T2 (GREEN): Implement max content-length guard in MCP server stdio frame reader.
- T3 (GREEN): Add unit tests for guard boundary behavior (`max` and `max + 1`).
- T4 (VERIFY): Run targeted tests, fmt, and clippy.

## Tier Mapping
- Unit: T3
- Functional: T2
- Conformance: T1, T4
- Integration: T1, T4
- Regression: T4 (existing persistent framed session test)
- Property: N/A (no randomized invariant requirement)
- Contract/DbC: N/A (no contracts macros in touched modules)
- Snapshot: N/A (no snapshot updates)
- Fuzz: N/A (no new parser target introduced)
- Mutation: N/A (workspace mutation gate managed in CI)
- Performance: N/A (no throughput contract changes)
