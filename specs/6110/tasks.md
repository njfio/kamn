# Tasks: Issue #6110

## Ordered Tasks
- T1 (RED): Add state IO regression tests for atomic replace semantics and temp-file cleanup.
- T2 (GREEN): Implement atomic file-write helper (temp file + sync + rename) in `state_io.rs`.
- T3 (GREEN): Migrate JSON state persistence + relay spool drain truncation to atomic helper.
- T4 (VERIFY): Run `fmt`, `clippy`, and targeted `kamn-node` state/service-api tests.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: N/A (no route contract change)
- Conformance: T1, T4
- Integration: N/A (file-level persistence helper scope)
- Regression: T4
- Property: N/A (deterministic filesystem operation paths)
- Contract/DbC: N/A (no DbC macros)
- Snapshot: N/A (no snapshot artifacts)
- Fuzz: N/A (no parser/untrusted input change)
- Mutation: N/A (workspace mutation gate handled in CI)
- Performance: N/A (no throughput contract change)
