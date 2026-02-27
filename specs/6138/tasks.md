# Tasks: Issue #6138

## Ordered Tasks
- T1 (RED): Extend cargo-fuzz contract tests/docs markers to require the new target and observe failing state before implementation.
- T2 (Implementation): Add `kolme_flat_json_policy_parser` fuzz target and wire it into `fuzz/Cargo.toml`.
- T3 (Implementation): Add deterministic corpus seeds + replay metadata entry for the new target.
- T4 (GREEN): Update docs inventory markers and rerun contract tests.
- T5 (Verify): Run bounded fuzz smoke execution for the new target.

## Tier Mapping
- Unit: N/A
- Functional: T4
- Conformance: T1, T4
- Regression: T3, T4
- Integration: N/A
- Fuzz: T5
