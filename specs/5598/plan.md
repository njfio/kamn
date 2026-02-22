# Issue #5598 Plan - PRD Phase-6d Live Orchestration and Validation Execution Contracts

## Approach
1. Add RED tests for `live_execution` marker presence and deterministic canonical values.
2. Implement deterministic `live_execution` composition in run output.
3. Add phase-6d docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase6d_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase6d-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: completion marker drift breaks downstream consumers.
  - Mitigation: lock deterministic completion markers in conformance tests.
- Risk: incoherence with prior phase-6 contracts.
  - Mitigation: derive canonical values aligned to existing pass-state scaffolding markers.

## Interfaces / Contracts
- `live_execution.orchestration_status`
- `live_execution.validation_status`
- `live_execution.evidence_status`
- `live_execution.overall_status`

## ADR
- Not required for deterministic contract extension.
