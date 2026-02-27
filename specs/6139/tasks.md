# Tasks: Issue #6139

## Ordered Tasks
- T1 (RED): Add failing regression test for untracked shell-file contamination.
- T2 (GREEN): Switch surface counting to git-tracked-file enumeration.
- T3 (REFACTOR): Keep deterministic error paths/reason markers unchanged while simplifying file enumeration helpers.
- T4 (VERIFY): Run scoped policy tests plus `cargo fmt --check` and scoped `clippy`.
- T5 (CLOSE): Update issue/PR with AC-to-test evidence and measured shell-surface markers.

## Tier Mapping
- Unit: T1, T2, T4
- Functional: T2, T4
- Regression: T1, T4
- Conformance: T4, T5
- Integration: N/A (single-test-module policy implementation)
