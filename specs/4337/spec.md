# Spec — #4337 Subtask: Runtime Module-Boundary Checker with CI Smoke Parity Governance

Status: Implemented
Priority: P1
Parent: #4329
Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance

## Problem Statement

Runtime decomposition governance needs a deterministic module-boundary checker that remains ci-smoke cheap while preserving local-heavy deep-lane contracts.

## Scope

In scope:
- Deterministic runtime module-boundary checker integration in full-stack runtime contract script.
- Policy validation and reason normalization for module-boundary drift.
- CI/docs governance marker updates and conformance tests.

Out of scope:
- New deep-lane command surfaces.

## Acceptance Criteria

AC-1: Checker emits runtime module-boundary parity statuses and taxonomy markers.

AC-2: Policy checker fails closed with deterministic module-boundary reason mapping.

AC-3: CI smoke boundary markers remain explicit and stable.

AC-4: Runtime architecture and CI docs/tests reflect the new checker governance contract.

## Conformance Cases

- C-01 (AC-1, Functional): run-lane output/json includes module-boundary taxonomy and parity markers.
- C-02 (AC-2, Regression): policy output includes deterministic module-boundary reason value normalization.
- C-03 (AC-2, Regression): boundary-status tamper drill fails with deterministic boundary drift reason.
- C-04 (AC-4, Conformance): runtime/ci docs include and tests assert new marker contract lines.

## Success Signals

- Runtime module-boundary checker parity drift is deterministic and fail-closed.
- CI smoke governance markers remain bounded and auditable.
