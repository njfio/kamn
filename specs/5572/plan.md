# Issue #5572 Plan - PRD Phase-4e Orchestration Lifecycle Step-Record Contracts

## Approach
1. Add RED conformance tests for nested step-record fields and required INFRA_UP/AGENT_DEPLOY step markers.
2. Extend phase-result model with step-record struct and deterministic status labels.
3. Generate deterministic step records in run output for INFRA_UP/AGENT_DEPLOY using PRD section-11.2 action names.
4. Update docs/research and milestone index markers.
5. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase4e_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase4e-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: step-label drift from PRD wording.
  - Mitigation: conformance tests assert explicit PRD-aligned step markers.
- Risk: unstable JSON ordering.
  - Mitigation: deterministic rendering order for phase steps.

## Interfaces / Contracts
- `OrchestrationPhaseResult.steps` contains ordered step records.
- step-record schema: `step`, `status`, `detail`.
- INFRA_UP and AGENT_DEPLOY have deterministic step lists.

## ADR
- Not required for deterministic contract extension.
