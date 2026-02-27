# Spec: Issue #6125 - Task: [S-02] Split kamn-core into 4-6 focused crates

- Issue: #6125
- Status: Reviewed
- Type: task
- Priority: P2
- Area: backend
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6101

## Problem Statement
`kamn-core` currently concentrates many architectural domains in one crate. S-02 requires a focused split that improves layering and incremental compile boundaries while preserving runtime behavior.

## Scope
In scope:
- Deliver a bounded phase-1 decomposition that introduces at least one focused crate extracted from `kamn-core`.
- Preserve existing behavior through re-exports and compatibility tests.
- Add conformance/regression tests proving the new crate boundary is real and wired.

Out of scope:
- Full 4-6 crate decomposition in a single PR.
- Public API removals or breaking changes unrelated to boundary extraction.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: A focused crate extracted from `kamn-core` is introduced and consumed by `kamn-core`.
- AC-2: Regression/conformance tests verify the extracted boundary and behavior parity.
- AC-3: Issue closure includes measurable evidence and linked PR.

## Conformance Cases
- C-01 (Conformance, AC-1): Build graph shows new extracted crate and `kamn-core` dependency wiring.
- C-02 (Regression, AC-2): Existing behavior tests for migrated modules remain green after extraction.
- C-03 (Conformance, AC-2): New boundary contract test(s) assert symbols are sourced from extracted crate.
- C-04 (Conformance, AC-3): PR and issue log include RED/GREEN commands and measured verification outputs.

## Success Metrics / Observable Signals
- `cargo test -p <new-crate>` and affected `kamn-core` suites pass.
- Boundary contract tests pass and fail when wiring is removed.
- No API regressions in migrated module surfaces.
