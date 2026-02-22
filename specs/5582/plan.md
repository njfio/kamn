# Issue #5582 Plan - PRD Phase-4j Live Process Runtime Hardening Contracts

## Approach
1. Add RED conformance tests for runtime readiness markers and mode-aware status behavior.
2. Implement runtime readiness composition in `execute_run_contract`.
3. Add phase-4j docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase4j_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase4j-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: readiness JSON shape drift from parser/run constraints.
  - Mitigation: lock exact markers with conformance tests.
- Risk: MCP error-path behavior diverges from parser checks.
  - Mitigation: assert deterministic error string in tests.

## Interfaces / Contracts
- `runtime_readiness.kolme_binary`
- `runtime_readiness.agent_binary`
- `runtime_readiness.scenario_selection`
- `runtime_readiness.overall`
- status values: `PASS`, `FAIL`, `SKIP`

## ADR
- Not required for deterministic readiness contract extension.
