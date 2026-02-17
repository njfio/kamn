# Spec — #4239 Task: Replay-Integrity CI Smoke Governance

Status: Reviewed
Priority: P1
Parent: #4235
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

Crash-replay governance now has deterministic policy and evidence contracts, but fast-gate still needs a dedicated low-cost CI smoke checker that fails closed on marker drift and enforces heavy run-mode exclusion.

## Scope

In scope:
- Add a CI smoke convergence checker for sqlite crash-recovery replay-integrity governance.
- Enforce deterministic fail-closed reason taxonomy for marker drift and budget violations.
- Enforce run-mode heavy-lane exclusion in ci-fast-gate workflow and ci-tools fast mode.
- Update docs and docs-contract tests for checker markers and boundary policy.

Out of scope:
- Changing crash-recovery runtime logic.
- Enabling heavy run-mode replay drills in fast-gate.

## Acceptance Criteria

AC-1: CI smoke checker validates required replay-integrity smoke command composition in ci-tools fast mode.

AC-2: Checker fails closed when sqlite crash-recovery run-mode command leaks into fast-gate workflow or fast-mode ci-tools block.

AC-3: Strategy/plan docs contain deterministic marker taxonomy and boundary policy for this checker.

AC-4: CI/docs contract tests cover pass path and deterministic fail reasons for drift and budget overflow.

## Conformance Cases

- C-01 (AC-1, Functional): baseline repo state returns `status=pass`, `final_decision=GO`, and `sqlite_crash_recovery_ci_smoke_convergence_status=verified`.
- C-02 (AC-1, Regression): missing required fast-mode smoke command fails with deterministic `*_ci_smoke_composition_missing` reason.
- C-03 (AC-2, Regression): leaked sqlite crash-recovery run-mode command in ci-tools fast mode fails with deterministic run-mode leakage reason.
- C-04 (AC-2, Regression): leaked sqlite crash-recovery run-mode command in ci-fast-gate workflow fails with deterministic exclusion reason.
- C-05 (AC-4, Regression): `--max-seconds` over policy fails with deterministic seconds-exceeded reason.
- C-06 (AC-3, Docs): docs contract tests fail closed when required checker markers drift.
