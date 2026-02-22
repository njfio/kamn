# Issue #5580 Spec - PRD Phase-4i CI Live-Lane Integration and Hardening Contracts

- Status: Implemented
- Issue: #5580
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
PRD section 12 defines live E2E CI lanes, but repository workflow contracts are missing and therefore vulnerable to configuration drift.

## Scope
In scope:
- Add `.github/workflows/e2e-live.yml` with PRD-aligned trigger and lane markers.
- Include workflow lane contracts for:
  - `e2e-sdk-direct`
  - `e2e-mcp-agent`
  - `e2e-cli-smoke`
- Add deterministic contract tests that validate required workflow markers.
- Add phase-4i docs markers and milestone progression updates.

Out of scope:
- Secret provisioning.
- Production launch decisions.

## Acceptance Criteria
- AC-1: `.github/workflows/e2e-live.yml` exists with `schedule` + `workflow_dispatch` triggers.
- AC-2: workflow defines three PRD lane jobs with deterministic markers (`e2e-sdk-direct`, `e2e-mcp-agent`, `e2e-cli-smoke`).
- AC-3: lane steps include harness invocation markers for `sdk-direct`, `mcp-tau`, and `cli-scripted` modes.
- AC-4: RED->GREEN tests validate workflow marker presence and docs coherence.
- AC-5: phase-4i docs markers and milestone index are coherent.
- AC-6: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): `e2e-live.yml` includes schedule cron and `workflow_dispatch`.
- C-02 (AC-2): workflow includes `e2e-sdk-direct` job marker.
- C-03 (AC-2): workflow includes `e2e-mcp-agent` job marker.
- C-04 (AC-2): workflow includes `e2e-cli-smoke` job marker.
- C-05 (AC-3): workflow includes harness mode marker `--mode sdk-direct`.
- C-06 (AC-3): workflow includes harness mode marker `--mode mcp-tau`.
- C-07 (AC-3): workflow includes harness mode marker `--mode cli-scripted`.
- C-08 (AC-4): RED failures observed before implementation.
- C-09 (AC-4): GREEN passes observed after implementation.
- C-10 (AC-5): phase-4i docs marker artifact present.
- C-11 (AC-5): milestone index references #5580 as active issue.
- C-12 (AC-6): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- CI workflow contract for PRD live E2E lanes is present and test-guarded.
- Workflow drift is detectable via deterministic contract tests.
