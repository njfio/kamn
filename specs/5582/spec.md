# Issue #5582 Spec - PRD Phase-4j Live Process Runtime Hardening Contracts

- Status: Implemented
- Issue: #5582
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Run output includes integration config metadata, but lacks explicit runtime-readiness markers needed to gate future real process orchestration cutover.

## Scope
In scope:
- Add deterministic `runtime_readiness` object to run output.
- Include readiness markers for:
  - `kolme_binary`
  - `agent_binary`
  - `scenario_selection`
  - `overall`
- Use deterministic status labels: `PASS`, `FAIL`, `SKIP`.
- Ensure MCP modes require/pass `agent_binary` readiness and non-MCP modes mark it `SKIP`.
- Add RED->GREEN tests for readiness behavior across sdk-direct and mcp modes.
- Add phase-4j docs markers and milestone progression update.

Out of scope:
- Real process spawning/lifecycle.
- CI workflow changes.

## Acceptance Criteria
- AC-1: run output includes deterministic `runtime_readiness` object with required keys.
- AC-2: sdk-direct marks `agent_binary` readiness `SKIP` and overall readiness `PASS`.
- AC-3: mcp-tau with agent binary marks `agent_binary` readiness `PASS` and overall readiness `PASS`.
- AC-4: mcp modes without `agent_binary` fail with deterministic error.
- AC-5: RED->GREEN tests validate readiness behavior.
- AC-6: phase-4j docs markers and milestone index are coherent.
- AC-7: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): run output includes `runtime_readiness` object and all required keys.
- C-02 (AC-2): sdk-direct readiness has `agent_binary` status `SKIP` and `overall` `PASS`.
- C-03 (AC-3): mcp-tau readiness has `agent_binary` status `PASS` and `overall` `PASS`.
- C-04 (AC-4): mcp-any run without `agent_binary` returns deterministic error.
- C-05 (AC-5): RED failures observed before implementation.
- C-06 (AC-5): GREEN passes observed after implementation.
- C-07 (AC-6): phase-4j docs marker artifact present.
- C-08 (AC-6): milestone index references #5582 as active issue.
- C-09 (AC-7): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- `runtime_readiness` provides machine-readable rollout gating signals.
- Mode-aware readiness outcomes are deterministic and test-guarded.
