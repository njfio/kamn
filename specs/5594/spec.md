# Issue #5594 Spec - PRD Phase-6b Spawn Execution Contracts

- Status: Implemented
- Issue: #5594
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Phase-6a added deterministic spawn command planning, but run output still lacks explicit spawn execution result contracts for each process role. This blocks contract-level validation of execution status coherence before real live-process orchestration lands.

## Scope
In scope:
- Add deterministic `spawn_execution` object to run output.
- Required role keys:
  - `postgres`
  - `kolme`
  - `kamn_processor`
  - `kamn_listener`
  - `kamn_approver`
- Each role marker includes deterministic execution fields:
  - `status`
  - `timeline_ref`
  - `result`
- Ensure statuses are canonical and coherent with existing `spawn_timeline`/`spawn_plan` contracts.
- Add RED->GREEN tests for marker presence and deterministic coherence.
- Add phase-6b docs marker artifact and milestone progression update.

Out of scope:
- Executing real process spawns.
- Live network validation against running external binaries.

## Acceptance Criteria
- AC-1: run output includes deterministic `spawn_execution` object with required role keys.
- AC-2: execution status markers are canonical and coherent with timeline/order contracts.
- AC-3: RED->GREEN tests validate spawn-execution behavior.
- AC-4: phase-6b docs markers and milestone index are coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): run output includes `spawn_execution.postgres`.
- C-02 (AC-1): run output includes `spawn_execution.kolme`.
- C-03 (AC-1): run output includes `spawn_execution.kamn_processor`.
- C-04 (AC-1): run output includes `spawn_execution.kamn_listener`.
- C-05 (AC-1): run output includes `spawn_execution.kamn_approver`.
- C-06 (AC-2): each role includes canonical `status` marker.
- C-07 (AC-2): `timeline_ref` markers are coherent with `spawn_timeline` ordering.
- C-08 (AC-2): `result` markers are deterministic across runs/modes.
- C-09 (AC-3): RED failures observed before implementation.
- C-10 (AC-3): GREEN passes observed after implementation.
- C-11 (AC-4): phase-6b docs marker artifact present.
- C-12 (AC-4): milestone index references #5594 as active issue.
- C-13 (AC-5): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- `spawn_execution` markers are deterministic, machine-readable, and stable.
- Execution contract markers are coherent with existing phase-5/phase-6a runtime/timeline/plan contracts.
