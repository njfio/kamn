# Spec: Issue #6089 - Shell-Surface Reduction Wave 1 (Wave Wrapper Contract Harness Consolidation)

- Issue: #6089
- Status: Reviewed
- Type: task
- Priority: P1
- Area: devops
- Milestone: `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`
- Last Updated: 2026-02-26
- Parent: #6086

## Problem Statement
Shell surface remains oversized, and two shared wave-wrapper contract harness implementations are long shell scripts with substantial assertion/mutation boilerplate:
- `scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/ci/test_wave_wrapper_family_baseline_contract_impl.sh`

This surface can be reduced without changing command entrypoints by keeping the `.sh` scripts as compatibility wrappers and moving implementation bodies to Python.

## Scope
In scope:
- Keep existing `.sh` entrypoints and argument contracts for wave-wrapper harnesses.
- Move the two large implementation bodies above into Python counterparts.
- Preserve behavior parity for representative wave wrappers and runner-contract checks.
- Report shell-surface delta markers.

Out of scope:
- Changing wrapper filenames/dispatch entrypoints.
- Reworking non-wave shell families.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh` remains an entrypoint and delegates to Python implementation with equivalent behavior.
- AC-2: `scripts/ci/test_wave_wrapper_family_baseline_contract_impl.sh` remains an entrypoint and delegates to Python implementation with equivalent behavior.
- AC-3: Wave-wrapper trend and baseline harness checks pass for representative waves after migration.
- AC-4: Shell LOC decreases measurably and closure reports shell-surface markers.

## Conformance Cases
- C-01 (Conformance, AC-1): `bash scripts/ci/test_check_kolme_wave10_wrapper_family_budget_trend.sh` passes.
- C-02 (Conformance, AC-1): `bash scripts/ci/test_check_non_kolme_wave10_wrapper_family_budget_trend.sh` passes.
- C-03 (Conformance, AC-2): `bash scripts/ci/test_kolme_wave10_wrapper_family_baseline_contract.sh` passes.
- C-04 (Conformance, AC-2): `bash scripts/ci/test_non_kolme_wave10_wrapper_family_baseline_contract.sh` passes.
- C-05 (Regression, AC-3): `bash scripts/ci/test_kolme_wave_budget_trend_runner_contract.sh` and `bash scripts/ci/test_non_kolme_wave_budget_trend_runner_contract.sh` pass.
- C-06 (Conformance, AC-4): git diff shows net negative shell LOC and no wrapper entrypoint removals for the migrated pair.

## Success Metrics / Observable Signals
- Net shell LOC decreases for this bounded wave-wrapper harness slice.
- Existing wave wrapper command paths and runner contract checks remain stable.
