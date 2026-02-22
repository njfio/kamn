# Issue #5586 Plan - PRD Phase-5b Process Lifecycle State Contracts

## Approach
1. Add RED tests for required process_lifecycle keys and canonical values.
2. Implement `process_lifecycle` object in run output.
3. Add phase-5b docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase5b_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase5b-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: output schema drift for downstream parsers.
  - Mitigation: enforce explicit key/value markers via conformance tests.
- Risk: lifecycle state naming inconsistency.
  - Mitigation: use one canonical state (`planned`) across all services.

## Interfaces / Contracts
- `process_lifecycle.postgres`
- `process_lifecycle.kolme`
- `process_lifecycle.kamn_processor`
- `process_lifecycle.kamn_listener`
- `process_lifecycle.kamn_approver`

## ADR
- Not required for deterministic contract extension.
