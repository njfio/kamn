# Spec — Issue #4825

- Title: Subtask: introduce `scripts/lib/test_harness.sh` and migrate first 50 high-duplication tests
- Parent: #4814
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Introduce a shared shell test harness and migrate the first high-duplication wave-wrapper contract tests to shared implementations without changing lane behavior.

## Problem Statement

Wave-wrapper contract test families duplicated argument parsing, fixture precondition checks, and assertion helpers across multiple large scripts. That duplication increases shell LOC and makes policy updates error-prone.

## Scope

In scope:
- Add shared shell test harness helpers in `scripts/lib/test_harness.sh`.
- Add shared family-parameterized baseline and trend implementation scripts:
  - `scripts/ci/test_wave_wrapper_family_baseline_contract_impl.sh`
  - `scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh`
- Convert existing family entrypoints to thin wrappers that delegate to shared impls.
- Migrate one framework matrix test to harness helper precondition checks.
- Add migration contract test:
  - `scripts/lib/test_test_harness_migration_contract.sh`
- Refresh non-Kolme wave-trend soft-budget baseline fixture to match reduced shell LOC.

Out of scope:
- Migrating all CI script families in one change.
- Changing policy reason-code taxonomy/version semantics.

## Acceptance Criteria

- AC-1: Shared harness library exists and is sourced by migrated scripts.
- AC-2: Both family baseline/trend contract implementations execute through shared impls for Kolme and non-Kolme waves with unchanged contract outcomes.
- AC-3: Soft-budget baseline reflects post-migration LOC and associated checker tests pass.
- AC-4: Full CI tools regression suite remains green after migration.

## Conformance Cases

- C-01 (AC-1): `bash scripts/lib/test_test_harness_migration_contract.sh` passes and verifies harness sourcing in migrated scripts.
- C-02 (AC-2): migrated wrapper cohort passes:
  - `for wave in {1..19}; do bash scripts/ci/test_non_kolme_wave${wave}_wrapper_family_baseline_contract.sh; done`
  - `for wave in {1..19}; do bash scripts/ci/test_check_non_kolme_wave${wave}_wrapper_family_budget_trend.sh; done`
  - `for wave in 8 10 11; do bash scripts/ci/test_kolme_wave${wave}_wrapper_family_baseline_contract.sh; done`
  - `for wave in 8 10 11; do bash scripts/ci/test_check_kolme_wave${wave}_wrapper_family_budget_trend.sh; done`
  - `for wave in {10..19}; do bash scripts/framework/test_non_kolme_wave${wave}_lightweight_contract_lane_dispatch_wrapper_matrix.sh; done`
- C-03 (AC-3): `bash scripts/ci/test_check_non_kolme_wave_trend_test_loc_soft_budget.sh` passes with refreshed baseline fixture.
- C-04 (AC-4): `bash scripts/ci/test_ci_tools.sh` passes.

## Success Metrics / Signals

- Duplicated logic in 4 prior family-impl scripts is centralized into 2 shared impl scripts.
- Shared harness helper adoption is contract-tested.
- Non-Kolme trend test-lane baseline `total_shell_loc` is reduced and aligned with current scripts.
