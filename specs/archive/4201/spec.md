# Spec — Issue #4201

- Title: add CI smoke checker for full-stack convergence marker drift and local-heavy lane exclusion
- Parent: #4194
- Milestone: R27.22 End-to-end live validation harness and promotion evidence convergence
- Status: Implemented
- Priority: P1

## Problem Statement

Fast-gate currently relies on scattered local full-stack exclusion checks and does not have a single
composite checker that fails closed on drift in CI smoke command composition, marker taxonomy
declarations, and local-heavy run-mode exclusion boundaries.

## Scope

In scope:
- add a deterministic CI smoke convergence checker for local full-stack governance boundaries,
- add contract tests for checker pass/fail behavior and drift/tamper fixtures,
- integrate checker contract tests into `scripts/ci/test_ci_tools.sh` fast/full modes,
- update `docs/ci/strategy.md` with checker marker declarations and fail-closed reason taxonomy.

Out of scope:
- always-on run-mode harness execution in fast-gate,
- redesign of `ci-fast-gate.yml` workflow topology,
- release-governance docs closure updates in `docs/plans/*` (handled by sibling issue #4202).

## Acceptance Criteria

- AC-1: checker fails closed when local full-stack CI smoke command composition drifts in ci-tools
  fast mode.
- AC-2: checker fails closed when local-heavy run-mode local full-stack commands leak into
  `ci-fast-gate` or ci-tools fast mode.
- AC-3: checker enforces bounded CI smoke runtime budget (`--max-seconds <= 120`) with deterministic
  reason output.
- AC-4: CI strategy doc includes deterministic checker marker/taxonomy declarations used by checker.

## Conformance Cases

- C-01 (Functional): Baseline repository passes checker with `status=pass`,
  `final_decision=GO`, and `reason_codes_value=none`. (AC-1, AC-2, AC-3)
- C-02 (Regression): Missing local full-stack policy command in ci-tools fast mode fails checker
  with deterministic `local_full_stack_policy_ci_smoke_composition_missing`. (AC-1)
- C-03 (Regression): Leaked local full-stack run-mode command in ci-tools fast mode fails checker
  with deterministic `local_full_stack_run_mode_command_leaked_in_fast_mode`. (AC-2)
- C-04 (Regression): Leaked local full-stack run-mode command in `ci-fast-gate.yml` fails checker
  with deterministic `ci_fast_gate_local_full_stack_run_mode_not_excluded`. (AC-2)
- C-05 (Regression): `--max-seconds` over 120 fails checker with deterministic
  `local_full_stack_ci_smoke_seconds_exceeded`. (AC-3)
- C-06 (Docs contract): Missing strategy markers fail checker with deterministic
  `ci_strategy_local_full_stack_convergence_markers_missing`. (AC-4)

## Success Metrics / Signals

- New checker and contract test are executed in ci-tools fast/full modes.
- Convergence reason taxonomy is explicit and deterministic for all drift fixtures.
- Heavy local full-stack run-mode command paths remain excluded from fast-gate boundaries.
