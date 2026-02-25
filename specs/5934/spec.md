# Spec: Issue #5934 - Task: Reduce shell/python surface below policy ceiling and improve governance ratio

- Issue: #5934
- Status: Implemented
- Type: task
- Priority: P1
- Area: governance
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5919

## Problem Statement
Shell LOC exceeds hard ceiling and governance artifact churn dominates commit mix.

## Scope
In scope:
- Retire/merge redundant scripts, migrate high-value script logic to Rust lanes, and enforce ratio gates.

Out of scope:
- Removing essential CI safeguards without replacement.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Shell LOC is reduced below configured hard ceiling with measurable trend improvement.
- AC-2: Shell-to-Rust ratio improves and regression gates enforce non-backsliding.
- AC-3: Governance-commit ratio target (<50%) is tracked with policy checks and visible telemetry.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Shell LOC is reduced below configured hard ceiling with measurable trend improvement.
- C-02 (Functional, AC-2): Verify Shell-to-Rust ratio improves and regression gates enforce non-backsliding.
- C-03 (Functional, AC-3): Verify Governance-commit ratio target (<50%) is tracked with policy checks and visible telemetry.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: policy checker tests
- Functional: replacement Rust lanes for retired scripts
- Integration: CI workflows pass with reduced script surface
- Regression: shell ceiling and ratio checks enforced
- Performance: CI runtime budget remains within target

## Dependencies
- #5919

## Implementation Summary (2026-02-25)
- Added governance structural-coupling telemetry to `scripts/ci/generate_combined_shell_surface_trend_report.sh` by parsing review markers plus `docs/review/governance-structural-coupling.policy`.
- Extended `scripts/ci/check_combined_shell_surface_trend_policy.sh` to validate governance metrics and emit deterministic WARN/NO-GO reason codes for over-target governance ratios.
- Updated combined trend contract lanes and fixtures to remain fail-closed under the expanded report schema.

## AC Verification
- AC-1: ✅
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --ceiling-file .ci/shell-loc-hard-ceiling.env --output-json /tmp/ci-shell-loc-hard-ceiling.json`
- AC-2: ✅
  - `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json /tmp/ci-shell-rust-ratio-guardrail.json`
  - `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
- AC-3: ✅
  - `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
  - `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
- AC-4: ✅
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`

## TDD Evidence
- RED:
  - `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh` -> `expected governance_structural_coupling.status marker in combined trend report`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` -> `expected ... governance_structural_coupling_status=within_target marker`
- GREEN:
  - `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh` -> pass
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` -> pass
  - `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh` -> pass
  - `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh` -> pass
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` -> pass
