# Issue #5578 Spec - PRD Phase-4h Live Runtime Binary Config Contracts

- Status: Implemented
- Issue: #5578
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
The harness run contract currently omits PRD-aligned runtime binary configuration metadata required to bridge deterministic orchestration contracts to real-runtime integration.

## Scope
In scope:
- Extend `run` parser with `--kolme-binary <path>`.
- Extend `run` parser with `--agent-binary <path>` with mode-aware validation:
  - required for MCP modes (`mcp-tau`, `mcp-any`)
  - optional for non-MCP modes (`sdk-direct`, `cli-scripted`)
- Emit deterministic `integration_config` in run output:
  - `kolme_binary`
  - `agent_binary`
  - `agent_binary_required`
- Add RED->GREEN conformance tests for parser and run-output behavior.
- Add phase-4h docs markers and milestone progression update.

Out of scope:
- Real process spawning/lifecycle management.
- CI workflow changes.

## Acceptance Criteria
- AC-1: parser accepts `run` commands with `--kolme-binary` and mode-validates `--agent-binary`.
- AC-2: MCP modes fail parsing when `--agent-binary` is omitted.
- AC-3: run output includes deterministic `integration_config` with required fields.
- AC-4: RED->GREEN tests validate parser and output contracts.
- AC-5: phase-4h docs markers and milestone index are coherent.
- AC-6: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): parser accepts sdk-direct run with `--kolme-binary` and no `--agent-binary`.
- C-02 (AC-1): parser accepts mcp-tau run with both `--kolme-binary` and `--agent-binary`.
- C-03 (AC-2): parser rejects mcp-any run without `--agent-binary`.
- C-04 (AC-3): run output contains `integration_config.kolme_binary` and `integration_config.agent_binary_required`.
- C-05 (AC-3): run output contains deterministic `integration_config.agent_binary` marker for non-MCP modes.
- C-06 (AC-4): RED failure observed before implementation.
- C-07 (AC-4): GREEN passes observed after implementation.
- C-08 (AC-5): phase-4h docs marker artifact present.
- C-09 (AC-5): milestone index references #5578 as active issue.
- C-10 (AC-6): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- `run` command surface now carries explicit runtime binary config contracts.
- Output has deterministic `integration_config` markers that downstream live-runtime wiring can consume.
