# Issue #5602 Spec - PRD Phase-6 Runtime External Process Orchestration

- Status: Implemented
- Issue: #5602
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
The harness now supports guarded external execution integration but lacks deterministic role-level runtime orchestration records for external execution path progression.

## Scope
In scope:
- Add deterministic `runtime_orchestration` object in run output.
- Include role-level markers for external execution orchestration:
  - `postgres`
  - `kolme`
  - `kamn_processor`
  - `kamn_listener`
  - `kamn_approver`
- For each role emit deterministic orchestration fields:
  - `requested`
  - `status`
  - `detail`
- Ensure coherence with `runtime_external_execution` guard path.
- Add RED->GREEN tests and docs/milestone updates.

Out of scope:
- Full non-deterministic live process orchestration implementation.
- Protocol/wire-format changes.

## Acceptance Criteria
- AC-1: run output includes deterministic `runtime_orchestration` role markers.
- AC-2: role markers are coherent with external execution guard state.
- AC-3: RED->GREEN tests validate behavior.
- AC-4: docs/milestone markers are coherent.
- AC-5: quality gates pass.

## Conformance Cases
- C-01 (AC-1): run output includes `runtime_orchestration.postgres`.
- C-02 (AC-1): run output includes `runtime_orchestration.kolme`.
- C-03 (AC-1): run output includes `runtime_orchestration.kamn_processor`.
- C-04 (AC-1): run output includes `runtime_orchestration.kamn_listener`.
- C-05 (AC-1): run output includes `runtime_orchestration.kamn_approver`.
- C-06 (AC-2): non-external mode role markers are deterministic `requested=false` and `status=SKIP`.
- C-07 (AC-2): external mode role markers are deterministic `requested=true` and `status=PASS` when preflight succeeds.
- C-08 (AC-3): RED failures observed before implementation.
- C-09 (AC-3): GREEN passes observed after implementation.
- C-10 (AC-4): docs marker artifact present.
- C-11 (AC-4): milestone index references #5602 as active issue.
- C-12 (AC-5): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- Role-level runtime orchestration markers are deterministic and machine-readable.
- Marker semantics align with guarded external execution contracts.
