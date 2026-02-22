# Issue #5602 Plan - PRD Phase-6 Runtime External Process Orchestration

## Approach
1. Add RED tests for `runtime_orchestration` role markers and deterministic requested/status/detail semantics.
2. Implement `runtime_orchestration` object composition in run output tied to `external_execution` guard state.
3. Add runtime orchestration docs marker artifact + milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase6_runtime_orchestration_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase6-runtime-orchestration-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: orchestration marker semantics drift from guard state.
  - Mitigation: lock deterministic cross-marker assertions in conformance tests.
- Risk: regressions in existing phase-6 contracts.
  - Mitigation: full harness + regression suite gates.

## Interfaces / Contracts
- `runtime_orchestration.<role>.requested`
- `runtime_orchestration.<role>.status`
- `runtime_orchestration.<role>.detail`

## ADR
- Not required for this additive deterministic contract slice.
