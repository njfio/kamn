# Issue #5576 Plan - PRD Phase-4g Lifecycle Summary Aggregation Contracts

## Approach
1. Add RED conformance tests for lifecycle summary object presence and deterministic counters.
2. Implement lifecycle summary aggregation helpers from phase/step result arrays.
3. Integrate `lifecycle_summary` into run output JSON.
4. Add phase-4g docs markers and milestone progression update.
5. Run gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase4g_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase4g-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: aggregation drift from emitted record statuses.
  - Mitigation: derive counters directly from phase/step arrays and lock with tests.
- Risk: run output JSON growth breaks older string assertions.
  - Mitigation: keep previous markers unchanged and append summary field.

## Interfaces / Contracts
- `lifecycle_summary.phase_totals.{total,pass,fail,skip}`
- `lifecycle_summary.step_totals.{total,pass,fail,skip}`

## ADR
- Not required for deterministic contract extension.
