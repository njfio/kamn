# Issue #5568 Plan - PRD Phase-4c Harness Orchestration Phase-State Contracts

## Approach
1. Add RED tests enforcing phase inventory/order and run output phase markers.
2. Implement phase model (`OrchestrationPhase`) and canonical inventory function.
3. Implement deterministic phase progression report model for run command.
4. Integrate phase markers into `execute_run_contract` JSON output.
5. Add phase-4c docs marker artifact and milestone index progression updates.
6. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase4c_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase4c-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: phase markers drift from PRD naming.
  - Mitigation: tests use exact PRD labels and order.
- Risk: JSON output destabilization across updates.
  - Mitigation: deterministic rendering order and byte-equality tests.

## Interfaces / Contracts
- `all_orchestration_phases()` returns canonical sequence.
- Run output JSON includes:
  - `phase_count`
  - `phases` (ordered phase labels)

## ADR
- Not required for this deterministic contract modeling slice.
