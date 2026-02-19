# Spec — #4329 Task: Runtime Decomposition Checkpoints with Deterministic Parity Checker Governance

Status: Implemented
Priority: P1
Parent: #4325
Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance

## Problem Statement

Runtime decomposition needs deterministic module-boundary parity governance so extraction drift is detected cheaply in CI smoke paths and remains auditable in local-heavy deep lanes.

## Scope

In scope:
- Runtime module-boundary parity checker integration in local full-stack runtime contract lane.
- Deterministic reason taxonomy and fail-closed mapping for module-boundary drift.
- CI smoke/local-heavy boundary governance markers and docs synchronization.

Out of scope:
- Full runtime architecture rewrite.
- New runtime modes beyond current local full-stack integration contract lane.

## Acceptance Criteria

AC-1: Runtime module-boundary checker validates boundary parity markers and fails closed on drift.

AC-2: Drift is reason-mapped with deterministic module-boundary taxonomy markers.

AC-3: CI smoke parity boundary remains low-cost and local-heavy deep lane remains explicit opt-in.

AC-4: Runtime architecture and CI strategy docs reflect module-boundary taxonomy and marker contracts.

## Conformance Cases

- C-01 (AC-1, Unit/Functional): checker validates runtime module boundary markers for main/runtime_orchestration/daemon_phase/runtime_kolme_live.
- C-02 (AC-2, Regression): policy check rejects boundary drift with deterministic reason code mapping.
- C-03 (AC-3, Integration): local full-stack dry-run emits ci-smoke-compatible module-boundary parity markers and budget boundary status.
- C-04 (AC-4, Conformance): docs/tests assert module-boundary parity taxonomy and CI boundary markers.

## Success Signals

- Deterministic boundary drift reasons surface in policy output.
- Runtime parity checker governance remains green in ci-smoke dry-run paths.
- Docs contract tests remain synchronized with marker taxonomy.
