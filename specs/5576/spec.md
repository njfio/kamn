# Issue #5576 Spec - PRD Phase-4g Lifecycle Summary Aggregation Contracts

- Status: Implemented
- Issue: #5576
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Run output now includes mode-aware phase and step records, but lacks deterministic lifecycle summary counters needed for promotion gating and diagnostics.

## Scope
In scope:
- Add deterministic lifecycle summary counters to run output for:
  - phases: `total`, `pass`, `fail`, `skip`
  - steps: `total`, `pass`, `fail`, `skip`
- Aggregate counters from emitted phase and step statuses.
- Add RED->GREEN conformance tests for normal and fail-path summary behavior.
- Add phase-4g docs/milestone markers.

Out of scope:
- Real process orchestration.
- CI workflow changes.

## Acceptance Criteria
- AC-1: run output includes lifecycle summary object with phase + step totals.
- AC-2: summary counters deterministically reflect fail-path status changes.
- AC-3: RED->GREEN tests validate summary behavior.
- AC-4: phase-4g docs markers are present and coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): run output includes `lifecycle_summary.phase_totals` with required keys.
- C-02 (AC-1): run output includes `lifecycle_summary.step_totals` with required keys.
- C-03 (AC-1): normal sdk-direct run summary totals are deterministic and non-zero.
- C-04 (AC-2): controlled fail-path run increments phase fail and step fail totals.
- C-05 (AC-3): RED failures observed before implementation.
- C-06 (AC-3): GREEN passes observed after implementation.
- C-07 (AC-4): phase-4g docs marker artifact present and coherent.
- C-08 (AC-4): milestone index references #5576 as active slice.
- C-09 (AC-5): fmt/clippy/tests/regressions pass.

## Success Metrics / Observable Signals
- Lifecycle summary markers are machine-readable and deterministic for downstream gating logic.
