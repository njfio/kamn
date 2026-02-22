# Plan: #5690 Consolidate Harness Doc-Contract Test Files

## Approach
1. Capture baseline docs-contract file inventory/count.
2. Create grouped replacement files:
   - `phase_docs_contract.rs` (phase4-6 families)
   - `r52_r64_docs_contract.rs` (release-series families)
3. Move each assertion pair from legacy files into grouped files without changing checks.
4. Delete superseded per-file docs-contract tests.
5. Run harness suite and verify parity.

## Affected Modules
- `crates/kamn-e2e-harness/tests/*.rs` (docs-contract subset)
- `specs/5690/spec.md`
- `specs/5690/plan.md`
- `specs/5690/tasks.md`

## Risks and Mitigations
- Risk: accidental assertion omission during file moves.
- Mitigation: copy assertions verbatim and run full harness suite.

- Risk: test-name collisions.
- Mitigation: preserve original function names where possible.

## Interfaces / Contracts
- No production API changes.
- Test contract surface preserved:
  - phase docs markers
  - milestone index references
  - R52-R64 marker families

## ADR
- Not required (test-organization refactor only).
