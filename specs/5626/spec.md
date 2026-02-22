# Spec: #5626 Mode Execution Contract Markers

- Issue: #5626
- Milestone: R53 E2E Scenario Execution Activation
- Status: Reviewed
- Priority: P1

## Problem Statement
Run output currently provides scenario and live status data but does not explicitly expose a mode-parity contract object describing which driver executed scenarios and whether execution counts are coherent with scenario selection.

## Scope
### In Scope
- Add `mode_execution_contract` object to run output.
- Include deterministic fields: `mode`, `driver`, `selected_scenarios`, `executed_scenarios`, `status`.
- Validate mode-coherent driver markers for sdk/cli/mcp modes.

### Out of Scope
- Changes to scenario execution logic itself.
- Runtime orchestration/process behavior changes.
- New dependencies.

## Acceptance Criteria
### AC-1 Contract object presence
Given run output generation,
When `execute_run_contract` returns,
Then output contains top-level `mode_execution_contract` with stable fields.

### AC-2 Mode-driver coherence
Given modes `sdk-direct`, `cli-scripted`, and `mcp-tau`,
When run output is produced,
Then `mode_execution_contract.driver` is `sdk-direct-driver`, `cli-scripted-driver`, and `mcp-agent-driver` respectively.

### AC-3 Count parity
Given successful scenario execution,
When run output is produced,
Then `mode_execution_contract.executed_scenarios == scenario_count`.

### AC-4 Contract stability
Given existing runtime/live/evidence contracts,
When this change is applied,
Then those marker objects remain present and semantically unchanged.

## Conformance Cases
- C-01 (AC-1): evidence of `mode_execution_contract` object and required fields in run output.
- C-02 (AC-2): mode-driver mapping assertions for sdk-direct/cli-scripted/mcp-tau.
- C-03 (AC-3): executed scenario count equals selected/scenario_count on success.
- C-04 (AC-4): runtime/live/evidence marker regression checks remain green.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with new mode contract assertions.
- `cargo test -p kamn-e2e-harness` green.
