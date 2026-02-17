# Spec — Issue #4186

- Title: add ci smoke checker for upgrade compatibility-rehearsal marker drift and heavy-lane exclusions
- Parent: #4179
- Milestone: R27.21 Kolme cross-version upgrade compatibility governance
- Status: Implemented
- Priority: P1

## Problem Statement

Fast-gate lacks a dedicated low-cost checker that enforces deterministic upgrade compatibility marker
parity and prevents heavy upgrade-rehearsal replay commands from leaking into CI smoke surfaces.

## Scope

In scope:
- add a deterministic CI smoke convergence checker for upgrade compatibility-rehearsal governance,
- add contract tests that enforce checker pass/fail behavior for drift and leakage fixtures,
- integrate checker contract tests into `scripts/ci/test_ci_tools.sh` fast/full paths,
- update `docs/ci/strategy.md` with checker command surface, marker taxonomy, and fail-closed reasons.

Out of scope:
- always-on heavy upgrade-rehearsal replay execution in fast-gate,
- release-plan closure synchronization in `docs/plans/*` (handled by sibling issue #4187),
- workflow topology redesign.

## Acceptance Criteria

- AC-1: checker fails closed when upgrade compatibility CI smoke composition drifts in ci-tools fast mode.
- AC-2: checker fails closed when heavy upgrade-rehearsal replay commands leak into ci-tools fast mode or `ci-fast-gate` workflow.
- AC-3: checker enforces bounded CI smoke budget (`--max-seconds <= 120`) with deterministic reason output.
- AC-4: strategy doc contains deterministic checker markers and fail-closed reason taxonomy used by checker.

## Conformance Cases

- C-01 (Functional): baseline repository passes checker with `status=pass`, `final_decision=GO`, and `reason_codes_value=none`. (AC-1, AC-2, AC-3)
- C-02 (Regression): missing fork-compatibility evidence smoke command in fast mode fails with `upgrade_compatibility_fork_evidence_ci_smoke_composition_missing`. (AC-1)
- C-03 (Regression): missing fork-compatibility policy smoke command in fast mode fails with `upgrade_compatibility_fork_policy_ci_smoke_composition_missing`. (AC-1)
- C-04 (Regression): leaked `run_version_compatibility_replay_deep_lane.sh` command in ci-tools fast mode fails with `upgrade_compatibility_replay_command_leaked_in_fast_mode`. (AC-2)
- C-05 (Regression): leaked replay command in `ci-fast-gate.yml` fails with `ci_fast_gate_upgrade_compatibility_replay_command_not_excluded`. (AC-2)
- C-06 (Regression): `--max-seconds` over 120 fails with `upgrade_compatibility_ci_smoke_seconds_exceeded`. (AC-3)
- C-07 (Docs contract): missing strategy markers fail with `ci_strategy_upgrade_compatibility_convergence_markers_missing`. (AC-4)

## Success Metrics / Signals

- Checker and checker contract test execute in ci-tools fast/full modes.
- Reason taxonomy and fail-closed reason ordering are deterministic.
- Heavy upgrade-rehearsal replay command remains excluded from CI smoke surfaces.
