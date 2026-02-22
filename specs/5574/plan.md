# Issue #5574 Plan - PRD Phase-4f Mode-Aware Lifecycle Population Contracts

## Approach
1. Add RED conformance tests for mode-aware AGENT_DEPLOY step statuses and controlled fail-path behavior.
2. Extend lifecycle population builder to accept execution mode and deterministic fail-path markers.
3. Implement deterministic status rules:
   - `[MCP modes]` steps: `PASS` only in MCP modes; `SKIP` otherwise.
   - controlled fail-path marker sets targeted INFRA_UP health-check step + phase to `FAIL`.
4. Add phase-4f docs markers and milestone index progression update.
5. Run gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase4f_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase4f-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: status-rule complexity causes non-deterministic output.
  - Mitigation: make rules pure function of mode + marker path and lock via tests.
- Risk: accidental impact to previous phase contracts.
  - Mitigation: retain prior assertions and rerun full harness contract suite.

## Interfaces / Contracts
- Mode-aware lifecycle status population in `execute_run_contract`.
- Controlled fail-path marker: deterministic detection from run config input.

## ADR
- Not required for deterministic contract extension.
