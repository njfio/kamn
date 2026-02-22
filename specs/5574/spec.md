# Issue #5574 Spec - PRD Phase-4f Mode-Aware Lifecycle Population Contracts

- Status: Implemented
- Issue: #5574
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Phase-4e emits structured phase/step records but all lifecycle statuses are static placeholders. The harness needs deterministic mode-aware population rules and controlled deterministic failure-path markers.

## Scope
In scope:
- Add deterministic mode-aware lifecycle status population for `run` output.
- Add deterministic failure-path marker support for lifecycle records in controlled contract paths.
- Add RED->GREEN conformance tests for mode-aware status behavior.
- Add phase-4f docs/research markers and milestone index updates.

Out of scope:
- Real runtime process orchestration.
- CI workflow modifications.

## Acceptance Criteria
- AC-1: `run` output lifecycle statuses vary deterministically by execution mode.
- AC-2: deterministic controlled non-pass (`FAIL`) lifecycle markers are supported.
- AC-3: RED->GREEN tests validate mode-aware and fail-path behavior.
- AC-4: phase-4f docs/research markers are present and coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): in `sdk-direct` mode, `[MCP modes]` AGENT_DEPLOY steps are `SKIP`.
- C-02 (AC-1): in `mcp-tau` mode, `[MCP modes]` AGENT_DEPLOY steps are `PASS`.
- C-03 (AC-1): non-MCP AGENT_DEPLOY steps stay deterministic `PASS` across modes.
- C-04 (AC-2): controlled fail-path marker sets deterministic `FAIL` for targeted INFRA_UP check step.
- C-05 (AC-2): controlled fail-path marker propagates deterministic `FAIL` to INFRA_UP phase result status.
- C-06 (AC-3): RED failures observed before implementation.
- C-07 (AC-3): GREEN passes observed after implementation.
- C-08 (AC-4): phase-4f docs marker artifact present and coherent.
- C-09 (AC-4): milestone index references #5574 as active slice.
- C-10 (AC-5): fmt/clippy/tests/regressions pass.

## Success Metrics / Observable Signals
- Harness run output can represent deterministic mode-specific lifecycle outcomes without depending on real processes.
- Controlled failure-path contract makes negative lifecycle scenarios machine-verifiable in offline tests.
