# Issue #5596 Plan - PRD Phase-6c Live Process Execution Contracts

## Approach
1. Add RED tests for `live_process_execution` role markers and deterministic state/health/pid coherence.
2. Implement deterministic `live_process_execution` composition in run output.
3. Add phase-6c docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase6c_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase6c-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: state marker drift breaks downstream consumers.
  - Mitigation: lock explicit role/state/health/pid markers in conformance tests.
- Risk: incoherence with prior spawn contracts.
  - Mitigation: deterministic role mapping and canonical marker values tied to existing contracts.

## Interfaces / Contracts
- `live_process_execution.postgres.{state,pid,health}`
- `live_process_execution.kolme.{state,pid,health}`
- `live_process_execution.kamn_processor.{state,pid,health}`
- `live_process_execution.kamn_listener.{state,pid,health}`
- `live_process_execution.kamn_approver.{state,pid,health}`

## ADR
- Not required for deterministic contract extension.
