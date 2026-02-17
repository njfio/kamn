# Spec — #4404 Task: Telemetry-Validation Gate Convergence

Status: Reviewed
Priority: P1
Parent: #4400
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Telemetry promotion gating must fail closed when evidence links are partial, missing, or non-convergent, while preserving low-cost CI smoke and explicit local-heavy boundaries.

## Scope

In scope:
- Telemetry gate evidence-link completeness checks.
- Deterministic fail-closed reason taxonomy coverage for evidence convergence gaps.
- CI smoke/local-heavy boundary governance markers for this telemetry gate.
- Tests and docs parity updates.

Out of scope:
- External observability backend rollout.
- New deep-lane orchestration outside existing unified local-heavy lane.

## Acceptance Criteria

AC-1: Telemetry policy checks fail closed when evidence links are incomplete or partial.

AC-2: Telemetry policy checks fail closed when linked evidence does not converge with declared report contracts.

AC-3: Deterministic reason-taxonomy markers and normalized reason outputs remain stable across pass/fail paths.

AC-4: CI smoke/local-heavy boundary governance remains explicit and deterministic in lane/policy outputs and docs.

## Conformance Cases

- C-01 (AC-1, Functional): policy rejects run-mode reports with missing required evidence-link keys.
- C-02 (AC-1, Regression): policy rejects run-mode reports with partial artifact path wiring.
- C-03 (AC-2, Integration): policy rejects run-mode reports where linked artifact schemas/status markers drift from declared contracts.
- C-04 (AC-3, Functional): pass path emits stable reason taxonomy/version and normalized reason value markers.
- C-05 (AC-4, Docs): CI strategy docs include telemetry gate boundary and fail-closed reason markers.

