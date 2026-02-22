# Issue #5606 Plan - PRD Phase-6 Runtime External Validation Execution

## Approach
1. Add RED tests for runtime validation execution markers across disabled/enabled external execution paths.
2. Implement deterministic `runtime_validation_execution` marker composition in run output.
3. Add runtime validation docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase6_runtime_validation_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase6-runtime-validation-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: validation marker semantics drift from existing external/lifecycle/live-validation contracts.
  - Mitigation: lock explicit deterministic assertions for both disabled/enabled paths.
- Risk: regression in prior phase-6 marker outputs.
  - Mitigation: full harness + regression suite gates.

## Interfaces / Contracts
- `runtime_validation_execution.requested`
- `runtime_validation_execution.orchestration_contract`
- `runtime_validation_execution.lifecycle_contract`
- `runtime_validation_execution.live_validation_contract`
- `runtime_validation_execution.evidence_contract`
- `runtime_validation_execution.overall`

## ADR
- Not required for additive deterministic contract extension.
