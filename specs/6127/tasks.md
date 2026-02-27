# Tasks: Issue #6127

## Ordered Tasks
- T1 (RED): add regression test that would fail if the two transition maps diverge or contain duplicates.
- T2 (GREEN): replace dual transition maps with a single canonical edge table and wire both lookups to it.
- T3 (VERIFY): run scoped lifecycle tests plus `cargo fmt --check` and scoped `clippy`.
- T4 (CLOSE): publish AC-to-test evidence in PR/issue closure updates.

## Tier Mapping
- Unit: T1, T2, T3
- Functional: T3
- Regression: T1, T3
- Conformance: T3, T4
- Integration: N/A (single-module state-machine refactor)
