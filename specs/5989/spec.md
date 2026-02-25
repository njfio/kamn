# Spec: Issue #5989 - Reduce shell surface via duplication collapse and shared entrypoints

- Issue: #5989
- Status: Reviewed (agent-authored P1; implementation proceeding)
- Type: story
- Priority: P1
- Area: devops
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
Two CI shell test scripts (`test_check_kamn_node_main_rs_extraction_threshold.sh` and `test_check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh`) duplicate nearly identical logic, inflating shell LOC and maintenance burden.

## Scope
In scope:
- Extract shared extraction-threshold test logic into one common shell harness.
- Keep existing per-script entrypoints intact via thin wrappers.
- Preserve all existing validation semantics and reason-code expectations.

Out of scope:
- Rewriting checker implementations.
- Changing extraction-threshold policy semantics.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Duplicated shell test logic is centralized in one common harness.
- AC-2: Existing script entrypoints remain usable and behavior-compatible.
- AC-3: Targeted shell test scripts pass after consolidation.
- AC-4: Net shell LOC for this surface decreases.

## Conformance Cases
- C-01 (Unit, AC-1): Common harness script receives parameterized checker/marker values and executes the full test matrix.
- C-02 (Functional, AC-2): Both per-surface wrapper scripts execute successfully with unchanged expectations.
- C-03 (Regression, AC-3): Existing reason-code assertions remain identical for warn/fail/exception scenarios.
- C-04 (Performance/Surface, AC-4): Measured shell LOC delta for touched scripts is negative.

## Success Metrics / Observable Signals
- Targeted shell tests pass.
- Wrappers remain small and deterministic.
- Shell LOC reduction is measurable and documented in PR summary.
