# Issue #5592 Spec - PRD Phase-6a Spawn Command-Plan Contracts

- Status: Implemented
- Issue: #5592
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
The harness now emits readiness/runtime/lifecycle/timeline/validation contracts, but it lacks explicit spawn command-plan markers needed before introducing executable process orchestration.

## Scope
In scope:
- Add deterministic `spawn_plan` object to run output.
- Required command-template keys:
  - `postgres_cmd`
  - `kolme_cmd`
  - `kamn_processor_cmd`
  - `kamn_listener_cmd`
  - `kamn_approver_cmd`
- Ensure command templates are canonical and mode-coherent.
- Add RED->GREEN tests for marker presence and deterministic values.
- Add phase-6a docs marker artifact and milestone progression update.

Out of scope:
- Actually spawning processes.
- Running live network validation.

## Acceptance Criteria
- AC-1: run output includes deterministic `spawn_plan` object with required command-template keys.
- AC-2: command-template markers are canonical and mode-coherent.
- AC-3: RED->GREEN tests validate spawn-plan behavior.
- AC-4: phase-6a docs markers and milestone index are coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): run output includes `spawn_plan.postgres_cmd`.
- C-02 (AC-1): run output includes `spawn_plan.kolme_cmd`.
- C-03 (AC-1): run output includes `spawn_plan.kamn_processor_cmd`.
- C-04 (AC-1): run output includes `spawn_plan.kamn_listener_cmd`.
- C-05 (AC-1): run output includes `spawn_plan.kamn_approver_cmd`.
- C-06 (AC-2): spawn plan templates are deterministic for sdk-direct mode.
- C-07 (AC-2): spawn plan templates include mode-coherent execution-mode markers.
- C-08 (AC-3): RED failures observed before implementation.
- C-09 (AC-3): GREEN passes observed after implementation.
- C-10 (AC-4): phase-6a docs marker artifact present.
- C-11 (AC-4): milestone index references #5592 as active issue.
- C-12 (AC-5): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- `spawn_plan` command templates are machine-readable and deterministic.
- Mode-coherent command planning markers are present before executable orchestration.
