# Spec — Issue #4172

- Title: Subtask: update custody docs and drift-contract tests for zeroization and signer-rotation closure
- Parent: Parent task: #4164
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Implemented
- Priority: P1

## Objective

Synchronize custody/rotation closure markers across CI strategy and production plan docs, then
extend drift-contract tests so marker-taxonomy and low-cost CI boundary regressions fail closed.

## Problem Statement

Without explicit R27.20 closure marker parity checks in docs-contract tests, custody/rotation
governance can drift between CI scripts and documentation.

## Scope

In scope:
- add deterministic R27.20 custody/rotation closure markers to strategy and plan docs
- extend docs-contract coverage to require those markers
- align low-cost CI (`max-seconds=120`) and local-heavy (`max-seconds=900`) boundaries in docs

Out of scope:
- signer runtime logic changes
- rollout policy changes beyond docs/checker alignment

## Acceptance Criteria

- AC-1: `docs/ci/strategy.md` contains custody/rotation CI smoke convergence governance markers,
  commands, taxonomy version/csv, and boundary markers.
- AC-2: `docs/plans/2026-02-14-production-service-next-steps.md` contains R27.20 closure chain
  and deterministic custody/rotation convergence markers.
- AC-3: docs-contract tests fail closed when the new R27.20 markers are missing or drifted.
- AC-4: closure evidence explicitly states low-cost CI smoke coverage and local-heavy opt-in
  boundaries for signer rotation/failover commands.

## Conformance Cases

- C-01 (AC-1, Conformance): strategy doc includes deterministic marker set referenced by checker.
- C-02 (AC-2, Conformance): production next-steps plan includes R27.20 closure marker set.
- C-03 (AC-3, Regression): `bash scripts/ci/test_production_service_next_steps_contract.sh`
  fails when R27.20 markers are removed.
- C-04 (AC-3, Functional): `bash scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh`
  fails on strategy/plan drift fixtures.
- C-05 (AC-4, Functional): checker and docs output/declare `120` CI smoke and `900` local-heavy
  boundary markers.

## Success Metrics / Signals

- Docs and checker marker sets remain aligned and machine-checked.
- R27.20 closure evidence is deterministic, taxonomy-versioned, and fail-closed on drift.
- Local-heavy boundary leakage remains explicitly prohibited in fast-gate surfaces.
