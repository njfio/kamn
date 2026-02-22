# Issue #5584 Spec - PRD Phase-5a Process Runtime Inventory Contracts

- Status: Implemented
- Issue: #5584
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Run output includes integration and runtime readiness markers, but lacks explicit process-runtime inventory markers required before real process spawn orchestration can be safely introduced.

## Scope
In scope:
- Add deterministic `process_runtime` object to run output.
- Required keys:
  - `kolme_runtime`
  - `kamn_nodes_runtime`
  - `agent_runtime`
  - `spawn_strategy`
- Add mode-aware `agent_runtime` labels:
  - `sdk-direct` -> `sdk-direct`
  - `cli-scripted` -> `cli-scripted`
  - `mcp-*` -> `mcp-agent`
- Add RED->GREEN tests for runtime inventory behavior.
- Add phase-5a docs marker artifact and milestone progression update.

Out of scope:
- Real process spawning/lifecycle management.
- Networked live execution.

## Acceptance Criteria
- AC-1: run output includes deterministic `process_runtime` object with required keys.
- AC-2: `agent_runtime` marker is mode-aware and deterministic.
- AC-3: RED->GREEN tests validate runtime inventory behavior.
- AC-4: phase-5a docs markers and milestone index are coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): run output includes `process_runtime.kolme_runtime`.
- C-02 (AC-1): run output includes `process_runtime.kamn_nodes_runtime`.
- C-03 (AC-1): run output includes `process_runtime.spawn_strategy`.
- C-04 (AC-2): sdk-direct emits `process_runtime.agent_runtime="sdk-direct"`.
- C-05 (AC-2): cli-scripted emits `process_runtime.agent_runtime="cli-scripted"`.
- C-06 (AC-2): mcp-tau emits `process_runtime.agent_runtime="mcp-agent"`.
- C-07 (AC-3): RED failures observed before implementation.
- C-08 (AC-3): GREEN passes observed after implementation.
- C-09 (AC-4): phase-5a docs marker artifact present.
- C-10 (AC-4): milestone index references #5584 as active issue.
- C-11 (AC-5): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- `process_runtime` inventory markers are machine-readable and deterministic.
- Mode-specific agent-runtime semantics are test-guarded before spawn implementation.
