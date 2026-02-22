# Issue #5586 Spec - PRD Phase-5b Process Lifecycle State Contracts

- Status: Implemented
- Issue: #5586
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Runtime inventory markers are present, but run output still lacks explicit per-service process lifecycle state markers needed before introducing real process spawn orchestration.

## Scope
In scope:
- Add deterministic `process_lifecycle` object to run output.
- Required service keys:
  - `postgres`
  - `kolme`
  - `kamn_processor`
  - `kamn_listener`
  - `kamn_approver`
- Use canonical deterministic lifecycle state value: `planned`.
- Add RED->GREEN tests for lifecycle marker presence/values.
- Add phase-5b docs markers and milestone progression update.

Out of scope:
- Real process spawning.
- Live networked execution.

## Acceptance Criteria
- AC-1: run output includes `process_lifecycle` object with required service keys.
- AC-2: service markers are deterministic and canonical (`planned`).
- AC-3: RED->GREEN tests validate lifecycle marker behavior.
- AC-4: phase-5b docs markers and milestone index are coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): run output includes `process_lifecycle.postgres`.
- C-02 (AC-1): run output includes `process_lifecycle.kolme`.
- C-03 (AC-1): run output includes `process_lifecycle.kamn_processor`.
- C-04 (AC-1): run output includes `process_lifecycle.kamn_listener`.
- C-05 (AC-1): run output includes `process_lifecycle.kamn_approver`.
- C-06 (AC-2): all process_lifecycle service values are `planned`.
- C-07 (AC-3): RED failures observed before implementation.
- C-08 (AC-3): GREEN passes observed after implementation.
- C-09 (AC-4): phase-5b docs marker artifact present.
- C-10 (AC-4): milestone index references #5586 as active issue.
- C-11 (AC-5): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- `process_lifecycle` markers are deterministic and machine-readable.
- Service-level lifecycle mapping exists before spawn implementation.
