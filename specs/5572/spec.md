# Issue #5572 Spec - PRD Phase-4e Orchestration Lifecycle Step-Record Contracts

- Status: Implemented
- Issue: #5572
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Phase-4d introduced top-level `phase_results`, but the model still lacks structured step records needed to represent PRD section-11.2 lifecycle actions within each phase.

## Scope
In scope:
- Add deterministic step-record model nested under phase results:
  - `step`
  - `status`
  - `detail`
- Populate deterministic step-record placeholders for `INFRA_UP` and `AGENT_DEPLOY` phases using PRD section-11.2 action lists.
- Extend run output contract to include nested `steps`.
- Add RED->GREEN conformance tests and phase-4e docs/milestone markers.

Out of scope:
- Real process execution behavior.
- CI workflow changes.

## Acceptance Criteria
- AC-1: run output `phase_results` entries include structured `steps` arrays.
- AC-2: `INFRA_UP` and `AGENT_DEPLOY` expose deterministic step markers aligned to PRD section 11.2 actions.
- AC-3: RED->GREEN tests validate step-record model/output contracts.
- AC-4: phase-4e docs/research markers are present and coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): phase-result entries include `steps`.
- C-02 (AC-1): each step entry includes `step`, `status`, `detail`.
- C-03 (AC-2): `INFRA_UP` includes deterministic markers for postgres/kolme/kamn/health/discovery actions.
- C-04 (AC-2): `AGENT_DEPLOY` includes deterministic markers for key generation, registration, MCP health, and evidence capture actions.
- C-05 (AC-3): RED failures are observed before implementation.
- C-06 (AC-3): GREEN passes are observed after implementation.
- C-07 (AC-4): phase-4e docs marker artifact exists and includes status markers.
- C-08 (AC-4): milestone index references #5572 as active slice.
- C-09 (AC-5): fmt/clippy/tests/regressions pass.

## Success Metrics / Observable Signals
- Run output now provides machine-readable per-phase lifecycle step markers suitable for future live execution population.
