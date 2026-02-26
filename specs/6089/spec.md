# Spec: Issue #6089 - Shell-Surface Reduction Wave 1 (Non-Kolme Wrapper Dedup)

- Issue: #6089
- Status: Reviewed
- Type: task
- Priority: P1
- Area: devops
- Milestone: `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`
- Last Updated: 2026-02-26
- Parent: #6086

## Problem Statement
The shell surface remains above the hard ceiling and contains heavy duplication. A single duplicated wrapper family contributes disproportionate LOC: 105 shell scripts are exact copies of `scripts/framework/run_non_kolme_contract_lane_dispatch.sh` (about 12.7K duplicate lines), inflating maintenance cost without increasing capability.

## Scope
In scope:
- Replace duplicated non-kolme dispatch wrappers with thin delegating wrappers that preserve command-path compatibility.
- Preserve wrapper invocation behavior and manifest resolution behavior.
- Measure and report shell/rust ratio delta markers in PR and closure.

Out of scope:
- Removing wrapper paths from command surface.
- Rewriting the manifest resolver/dispatcher semantics.
- Additional shell families beyond this duplicate set.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: The identified duplicate wrapper family is reduced to thin compatibility wrappers while preserving wrapper filenames/entrypoints.
- AC-2: Dispatcher resolution behavior remains unchanged for migrated wrappers.
- AC-3: Shell LOC decreases measurably and closure reports actual shell/rust ratio markers.
- AC-4: Stale-reference and wrapper parity checks pass for the migrated surface.

## Conformance Cases
- C-01 (Conformance, AC-1): all files in the migrated duplicate family delegate to `scripts/framework/run_non_kolme_contract_lane_dispatch.sh --lane-wrapper <basename>`.
- C-02 (Functional, AC-2): representative wrappers resolve and dispatch successfully with unchanged manifest lookup behavior.
- C-03 (Conformance/Regression, AC-3): measured post-change markers include `shell_loc_delta_actual < 0`, `rust_loc_delta_actual >= 0`, and explicit ratio delta reporting.
- C-04 (Regression, AC-4): stale-reference and wrapper-parity policy lanes pass for migrated wrappers.

## Success Metrics / Observable Signals
- Net shell LOC decreases by at least 10K in wave 1 from this family.
- No wrapper path removals are required for callers/CI workflows.
- Shell-surface DoD markers are present and internally consistent.
