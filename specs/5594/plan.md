# Issue #5594 Plan - PRD Phase-6b Spawn Execution Contracts

## Approach
1. Add RED tests for `spawn_execution` role markers and deterministic status/timeline/result coherence.
2. Implement deterministic `spawn_execution` composition in run output.
3. Add phase-6b docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase6b_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase6b-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: execution-marker drift breaks downstream consumers.
  - Mitigation: lock deterministic role/status/timeline/result markers in conformance tests.
- Risk: timeline incoherence with prior contracts.
  - Mitigation: enforce explicit `timeline_ref` expectations tied to existing phase-5c markers.

## Interfaces / Contracts
- `spawn_execution.postgres.{status,timeline_ref,result}`
- `spawn_execution.kolme.{status,timeline_ref,result}`
- `spawn_execution.kamn_processor.{status,timeline_ref,result}`
- `spawn_execution.kamn_listener.{status,timeline_ref,result}`
- `spawn_execution.kamn_approver.{status,timeline_ref,result}`

## ADR
- Not required for deterministic contract extension.
