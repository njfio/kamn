# Issue #5600 Spec - PRD Phase-6 Runtime External Execution Integration

- Status: Implemented
- Issue: #5600
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
The harness currently emits deterministic phase-6 contracts, but it has no guarded runtime external execution integration path. We need an explicit integration toggle with deterministic preflight checks and fail-fast errors to bridge contract scaffolding toward real runtime execution.

## Scope
In scope:
- Add explicit run flag: `--enable-external-execution`.
- Add deterministic runtime contract object: `runtime_external_execution`.
- Add preflight checks when external execution is enabled:
  - Kolme binary path must exist.
  - MCP modes require agent binary path to exist.
- Keep default behavior deterministic and contract-only when flag is not enabled.
- Add RED->GREEN tests for parser, guarded path, fail-fast errors, and docs/milestone markers.
- Add phase-6 runtime integration docs artifact and milestone progression update.

Out of scope:
- Full real process orchestration implementation.
- Chain-backed live validation runtime execution.

## Acceptance Criteria
- AC-1: `run` command parser accepts `--enable-external-execution` and maps deterministic config state.
- AC-2: run output includes deterministic `runtime_external_execution` contract markers.
- AC-3: missing-runtime preconditions fail deterministically with explicit errors.
- AC-4: RED->GREEN tests validate guarded path behavior.
- AC-5: phase-6 runtime integration docs markers and milestone index are coherent.
- AC-6: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): parser accepts `--enable-external-execution` for run command.
- C-02 (AC-2): default run output includes `runtime_external_execution` markers with contract-only status.
- C-03 (AC-2): enabled run output includes `runtime_external_execution` markers with external-runtime status.
- C-04 (AC-3): enabling external execution with missing Kolme binary returns deterministic preflight error.
- C-05 (AC-3): enabling external execution in MCP mode with missing agent binary returns deterministic preflight error.
- C-06 (AC-4): RED failures observed before implementation.
- C-07 (AC-4): GREEN passes observed after implementation.
- C-08 (AC-5): phase-6 runtime integration docs marker artifact present.
- C-09 (AC-5): milestone index references #5600 as active issue.
- C-10 (AC-6): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- External execution integration is explicit and guarded.
- Deterministic contract behavior remains stable when external execution is disabled.
- Preflight failures are machine-readable and deterministic.
