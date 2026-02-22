# Issue #5590 Plan - PRD Phase-5d Live Validation Summary Contracts

## Approach
1. Add RED tests for `live_validation` marker presence and deterministic values.
2. Implement deterministic `live_validation` object in run output.
3. Add phase-5d docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase5d_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase5d-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: final summary contract drifting from deterministic baseline.
  - Mitigation: hard assertions on all summary fields in conformance tests.
- Risk: output-schema compatibility drift.
  - Mitigation: append-only JSON contract extension.

## Interfaces / Contracts
- `live_validation.expected_checks`
- `live_validation.completed_checks`
- `live_validation.status`

## ADR
- Not required for deterministic contract extension.
