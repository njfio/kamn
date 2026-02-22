# Issue #5592 Plan - PRD Phase-6a Spawn Command-Plan Contracts

## Approach
1. Add RED tests for `spawn_plan` keys and canonical/mode-coherent template values.
2. Implement deterministic `spawn_plan` composition in run output.
3. Add phase-6a docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase6a_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase6a-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: spawn command template drift breaks downstream orchestration consumers.
  - Mitigation: lock explicit template markers in conformance tests.
- Risk: mode mapping inconsistency.
  - Mitigation: assert mode-specific marker presence in tests.

## Interfaces / Contracts
- `spawn_plan.postgres_cmd`
- `spawn_plan.kolme_cmd`
- `spawn_plan.kamn_processor_cmd`
- `spawn_plan.kamn_listener_cmd`
- `spawn_plan.kamn_approver_cmd`

## ADR
- Not required for deterministic contract extension.
