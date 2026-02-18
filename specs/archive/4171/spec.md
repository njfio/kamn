# Spec — Issue #4171

- Title: Subtask: add ci smoke checker for custody-rotation marker lineage with local-heavy exclusions
- Parent: Parent task: #4164
- Milestone: R27.20 Secret material zeroization and signer-rotation governance
- Status: Implemented
- Priority: P1

## Objective

Add a deterministic CI smoke convergence checker that fails closed when custody/rotation marker
lineage drifts or when local-heavy signer-rotation commands leak into fast-gate surfaces.

## Problem Statement

Custody and signer-rotation governance markers are currently spread across scripts and docs
without a dedicated CI smoke convergence checker, so marker drift and local-heavy boundary
regressions can slip through.

## Scope

In scope:
- add a new custody/rotation CI smoke convergence checker under `scripts/ci`
- add deterministic contract tests for baseline pass and fail-closed drift fixtures
- wire checker tests into `scripts/ci/test_ci_tools.sh` fast/full suites

Out of scope:
- backend signer runtime behavior changes
- always-on heavy rotation drill execution in PR fast-gate

## Acceptance Criteria

- AC-1: checker validates required CI smoke composition for custody/rotation governance in
  ci-tools fast mode.
- AC-2: checker fails closed when local-heavy rotation/failover commands appear in ci-tools fast
  mode or ci-fast-gate workflow.
- AC-3: checker enforces strategy/plan marker lineage parity for R27.20 closure markers.
- AC-4: checker emits deterministic taxonomy markers and JSON report fields on both pass and fail.

## Conformance Cases

- C-01 (AC-1, Functional): baseline run passes.
  - `python3 scripts/ci/check_custody_rotation_ci_smoke_convergence.py --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --strategy-doc docs/ci/strategy.md --plan-doc docs/plans/2026-02-14-production-service-next-steps.md --max-seconds 120`
- C-02 (AC-1, Regression): missing custody smoke command fixture fails with deterministic
  `*_ci_smoke_composition_missing` reason code.
- C-03 (AC-2, Regression): leaked local-heavy command fixture fails with deterministic fast-gate
  exclusion reason code.
- C-04 (AC-3, Conformance): strategy/plan marker drift fixtures fail with deterministic doc-drift
  reason codes.
- C-05 (AC-4, Unit/Functional): checker outputs deterministic taxonomy/version/csv markers and
  report schema fields.

## Success Metrics / Signals

- Fast-mode CI includes custody/rotation smoke convergence tests.
- Marker drift and local-heavy leakage regressions fail closed with deterministic reason codes.
- Strategy/plan/docs remain synchronized with checker contract markers.
