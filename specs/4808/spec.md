# Spec — Issue #4808

- Title: Story: collapse test/matrix/json shell boilerplate into reusable harnesses
- Parent: Parent epic: #4806
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Execute phases 3-5 by parameterizing wave scripts, introducing shared test harness utilities, and eliminating manual JSON output duplication.

## Problem Statement

Test and output boilerplate dominates shell LOC and increases drift risk between near-identical scripts.

## Scope

In scope:
- wave/matrix script parameterization
- test harness introduction and migration
- common JSON helper rollout

Out of scope:
- changing evidence schemas themselves
- removing contract-lane validation semantics

## Acceptance Criteria

- AC-1: Wave/matrix duplicates are replaced by parameterized runners with equivalent coverage.
- AC-2: Harness migration cuts repeated test setup/assert code while preserving deterministic failures.
- AC-3: Manual JSON emission footprint is materially reduced via shared helper calls.

## Conformance Cases

- C-01 (AC-1): parameterized wave/matrix runner migrations from task `#4813` merged and validated (`PR #4840`, `PR #4841`).
- C-02 (AC-2): harness migration contracts pass (`bash scripts/lib/test_test_harness_migration_contract.sh`) from task `#4814` / subtask `#4825` (`PR #4842`).
- C-03 (AC-3): JSON helper migration contracts pass (`bash scripts/lib/test_json_write_helper_migration_contract.sh`) from task `#4814` / subtask `#4826` (`PR #4843`).
- C-04 (AC-1..AC-3): `bash scripts/ci/test_ci_tools.sh` passes after migrations.

## Success Metrics / Signals

- Wave/matrix duplicate logic collapsed into parameterized runners.
- Shared harness and JSON helper utilities deployed and adopted across migrated cohorts.
- Full CI regression suite remained green across merged slices.
