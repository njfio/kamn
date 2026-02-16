# Tasks: Issue #4347

Status: Completed
Issue: #4347

## Ordered Tasks

T1 (RED):
- Add missing-marker and over-budget fail-closed assertions in non-Kolme wave wrapper trend test harness.

T2 (Capture RED Evidence):
- Run:
  - `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`

## TDD Evidence

- RED command/output:
  - `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`
    - Failed before implementation because required taxonomy markers were missing and runtime-budget overrun contract path was unsupported.

- Regression summary:
  - Shared non-Kolme wave trend test harness now enforces taxonomy marker and runtime-budget contracts.
