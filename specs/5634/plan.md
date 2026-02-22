# Plan: #5634 EVIDENCE Step Inventory Parity

## Approach
1. Add RED tests in `command_contract.rs` for six-step evidence inventory and fail-path propagation.
2. Implement evidence-step expansion in `crates/kamn-e2e-harness/src/lib.rs` with deterministic details and status wiring from `evidence_status`.
3. Update lifecycle total assertions and rerun crate verification gates.
4. Add R55 docs/research marker artifact and docs contract test.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/*docs_contract.rs` (new R55 docs contract)
- `docs/research/`

## Risks and Mitigations
- Risk: Step-total assertion drift across existing tests.
  - Mitigation: Update only evidence-coupled totals and run full command contract suite.
- Risk: Evidence fail-path semantics become ambiguous.
  - Mitigation: Explicitly assert fail statuses on evidence-sensitive steps only.
- Risk: Regressions in non-evidence markers.
  - Mitigation: run full crate tests and keep non-evidence builders untouched.

## Interfaces/Contracts
- No schema shape changes.
- Existing `phase_results.steps` contract is preserved with expanded EVIDENCE content.
