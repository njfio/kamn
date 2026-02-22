# Issue #5604 Plan - PRD Phase-6 Runtime External Lifecycle Execution

## Approach
1. Add RED tests for lifecycle transition markers across disabled/enabled external execution paths.
2. Implement deterministic `runtime_lifecycle_execution` role transition composition in run output.
3. Add runtime lifecycle docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase6_runtime_lifecycle_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase6-runtime-lifecycle-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: lifecycle transition semantics drift from existing guard/orchestration contracts.
  - Mitigation: lock explicit deterministic transition assertions for both disabled/enabled paths.
- Risk: regression in existing phase-6 contract outputs.
  - Mitigation: full harness + regression suite gates.

## Interfaces / Contracts
- `runtime_lifecycle_execution.<role>.init`
- `runtime_lifecycle_execution.<role>.spawn`
- `runtime_lifecycle_execution.<role>.health_check`
- `runtime_lifecycle_execution.<role>.ready`

## ADR
- Not required for additive deterministic contract extension.
