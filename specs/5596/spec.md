# Issue #5596 Spec - PRD Phase-6c Live Process Execution Contracts

- Status: Implemented
- Issue: #5596
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Phase-6b added deterministic spawn execution markers, but run output still lacks explicit live process execution state markers that represent role-level runtime state/health snapshots.

## Scope
In scope:
- Add deterministic `live_process_execution` object to run output.
- Required role keys:
  - `postgres`
  - `kolme`
  - `kamn_processor`
  - `kamn_listener`
  - `kamn_approver`
- Each role marker includes deterministic fields:
  - `state`
  - `pid`
  - `health`
- Ensure coherence with existing process/timeline/spawn contracts.
- Add RED->GREEN tests for marker presence and deterministic coherence.
- Add phase-6c docs marker artifact and milestone progression update.

Out of scope:
- Real process lifecycle execution against external binaries.
- Full live network validation execution.

## Acceptance Criteria
- AC-1: run output includes deterministic `live_process_execution` object with required role markers.
- AC-2: role state/health markers are canonical and contract-coherent.
- AC-3: RED->GREEN tests validate live-process-execution behavior.
- AC-4: phase-6c docs markers and milestone index are coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): run output includes `live_process_execution.postgres`.
- C-02 (AC-1): run output includes `live_process_execution.kolme`.
- C-03 (AC-1): run output includes `live_process_execution.kamn_processor`.
- C-04 (AC-1): run output includes `live_process_execution.kamn_listener`.
- C-05 (AC-1): run output includes `live_process_execution.kamn_approver`.
- C-06 (AC-2): each role includes canonical `state` marker.
- C-07 (AC-2): each role includes canonical `health` marker.
- C-08 (AC-2): each role includes deterministic `pid` marker.
- C-09 (AC-3): RED failures observed before implementation.
- C-10 (AC-3): GREEN passes observed after implementation.
- C-11 (AC-4): phase-6c docs marker artifact present.
- C-12 (AC-4): milestone index references #5596 as active issue.
- C-13 (AC-5): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- `live_process_execution` markers are deterministic and machine-readable.
- Role state/health snapshots remain coherent with prior phase-5/phase-6 contracts.
