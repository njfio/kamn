# Issue #5606 Spec - PRD Phase-6 Runtime External Validation Execution

- Status: Implemented
- Issue: #5606
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
The harness now exposes deterministic runtime external execution, orchestration, and lifecycle markers, but it lacks explicit deterministic runtime validation execution markers that summarize external validation contract state coherently.

## Scope
In scope:
- Add deterministic `runtime_validation_execution` object in run output.
- Include validation marker fields:
  - `requested`
  - `orchestration_contract`
  - `lifecycle_contract`
  - `live_validation_contract`
  - `evidence_contract`
  - `overall`
- Keep marker semantics coherent with external execution state.
- Add RED->GREEN tests and docs/milestone updates.

Out of scope:
- Full non-deterministic live validation runtime engine.
- Protocol/wire-format changes.

## Acceptance Criteria
- AC-1: run output includes deterministic `runtime_validation_execution` markers.
- AC-2: marker values are coherent with external execution state.
- AC-3: RED->GREEN tests validate behavior.
- AC-4: docs/milestone markers are coherent.
- AC-5: quality gates pass.

## Conformance Cases
- C-01 (AC-1): run output includes `runtime_validation_execution` object.
- C-02 (AC-1): object includes `requested` marker.
- C-03 (AC-1): object includes `orchestration_contract` marker.
- C-04 (AC-1): object includes `lifecycle_contract` marker.
- C-05 (AC-1): object includes `live_validation_contract` marker.
- C-06 (AC-1): object includes `evidence_contract` marker.
- C-07 (AC-1): object includes `overall` marker.
- C-08 (AC-2): external disabled path reports deterministic `SKIP` marker set.
- C-09 (AC-2): external enabled path reports deterministic `PASS` marker set after preflight.
- C-10 (AC-3): RED failures observed before implementation.
- C-11 (AC-4): docs marker artifact present and milestone index references #5606.
- C-12 (AC-5): required fmt/clippy/test gate commands pass.

## Success Metrics / Observable Signals
- Deterministic runtime validation execution markers are machine-readable and stable.
- Marker semantics remain coherent with existing phase-6 external marker family.
