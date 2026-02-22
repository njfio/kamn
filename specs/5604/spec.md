# Issue #5604 Spec - PRD Phase-6 Runtime External Lifecycle Execution

- Status: Implemented
- Issue: #5604
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
The harness now exposes deterministic runtime orchestration role markers, but it lacks explicit deterministic lifecycle transition records (`init`, `spawn`, `health_check`, `ready`) per role.

## Scope
In scope:
- Add deterministic `runtime_lifecycle_execution` object in run output.
- Include role markers for:
  - `postgres`
  - `kolme`
  - `kamn_processor`
  - `kamn_listener`
  - `kamn_approver`
- Per role include transitions:
  - `init`
  - `spawn`
  - `health_check`
  - `ready`
- Keep transition semantics coherent with external execution guard/orchestration state.
- Add RED->GREEN tests and docs/milestone updates.

Out of scope:
- Full non-deterministic live lifecycle execution engine.
- Protocol/wire-format changes.

## Acceptance Criteria
- AC-1: run output includes deterministic `runtime_lifecycle_execution` role transition markers.
- AC-2: transition markers are coherent with external execution state.
- AC-3: RED->GREEN tests validate behavior.
- AC-4: docs/milestone markers are coherent.
- AC-5: quality gates pass.

## Conformance Cases
- C-01 (AC-1): run output includes `runtime_lifecycle_execution.postgres`.
- C-02 (AC-1): run output includes `runtime_lifecycle_execution.kolme`.
- C-03 (AC-1): run output includes `runtime_lifecycle_execution.kamn_processor`.
- C-04 (AC-1): run output includes `runtime_lifecycle_execution.kamn_listener`.
- C-05 (AC-1): run output includes `runtime_lifecycle_execution.kamn_approver`.
- C-06 (AC-2): external disabled path has `SKIP` transitions.
- C-07 (AC-2): external enabled path has `PASS` transitions when preflight succeeds.
- C-08 (AC-3): RED failures observed before implementation.
- C-09 (AC-3): GREEN passes observed after implementation.
- C-10 (AC-4): docs marker artifact present.
- C-11 (AC-4): milestone index references #5604 as active issue.
- C-12 (AC-5): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- Deterministic lifecycle transition markers are machine-readable.
- Transition semantics remain coherent with guard and orchestration contracts.
