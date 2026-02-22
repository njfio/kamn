# Spec: #5631 TEARDOWN Phase Activation

- Issue: #5631
- Milestone: R54 E2E Evidence Phase Activation
- Status: Implemented
- Priority: P1

## Problem Statement
`phase_results` currently renders `TEARDOWN` as a static single-step `SKIP` placeholder. PRD section 11.2 defines explicit teardown lifecycle steps. The run contract needs deterministic teardown phase semantics aligned with the PRD lifecycle model.

## Scope
### In Scope
- Replace static teardown placeholder with deterministic teardown lifecycle steps aligned to PRD section 11.2.
- Compute TEARDOWN status/details from teardown step statuses.
- Keep run-output contract shape stable while activating existing teardown fields.

### Out of Scope
- Real process shutdown execution.
- Runtime process signaling/kill behavior.
- Changes to evidence artifact production logic.

## Acceptance Criteria
### AC-1 SDK/CLI teardown parity
Given non-MCP mode (`sdk-direct` or `cli-scripted`),
When run output is generated,
Then `TEARDOWN` phase includes PRD-aligned teardown steps with MCP stop step marked `SKIP` and remaining teardown steps `PASS`, and phase status resolves `PASS`.

### AC-2 MCP teardown parity
Given MCP mode (`mcp-tau` or `mcp-any`),
When run output is generated,
Then `TEARDOWN` includes MCP stop step with status `PASS`, core teardown steps are present and `PASS`, and phase status resolves `PASS`.

### AC-3 Lifecycle summary propagation
Given teardown activation,
When lifecycle summary is computed,
Then phase/step totals reflect TEARDOWN as active pass semantics instead of static skip placeholder.

### AC-4 Contract stability
Given existing runtime and live execution marker contracts,
When teardown activation is applied,
Then previously delivered runtime marker contracts remain stable and coherent.

## Conformance Cases
- C-01 (AC-1): sdk-direct output renders full teardown step inventory, MCP-stop `SKIP`, phase `PASS`.
- C-02 (AC-2): mcp-tau output renders full teardown step inventory, MCP-stop `PASS`, phase `PASS`.
- C-03 (AC-3): lifecycle totals update for normal/fail/evidence-fail paths with TEARDOWN pass activation.
- C-04 (AC-4): existing live/runtime contract tests remain green after teardown activation.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with teardown conformance assertions.
- `cargo test -p kamn-e2e-harness` remains green.
