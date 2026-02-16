# Plan: Issue #4452

Status: Completed
Issue: #4452

## Approach

1. Add RED decomposition drift assertions to
   `crates/kamn-node/tests/main_module_extraction_contract.rs` for `main_tests`
   module boundaries and anti-monolith guardrails.
2. Add RED docs contract coverage in a dedicated
   `crates/kamn-core/tests/testing_structure_docs.rs` test file.
3. Create/update `docs/testing/structure.md` with deterministic decomposition drift and
   structural budget governance markers required by the new docs contracts.
4. Execute targeted RED -> GREEN loops and scoped verification.

## Affected Modules

- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `crates/kamn-core/tests/testing_structure_docs.rs`
- `docs/testing/structure.md`
- `specs/4452/*`

## Risks and Mitigations

- Risk: string-based docs contracts can be brittle.
  - Mitigation: assert stable contract markers/commands only.
- Risk: decomposition guardrails could overconstrain healthy refactors.
  - Mitigation: assert module-boundary intent, not implementation internals.

## Interfaces / Contracts

- Source-level decomposition contract:
  - `main_tests.rs` should retain module boundary declarations.
  - `main_tests.rs` should not re-inline test bodies.
- Docs-level structure governance contract:
  - deterministic reason taxonomy marker
  - deterministic structural budget reason codes
  - deterministic command entrypoints for checks/lanes.

## ADR

Not required: no dependency/architecture decision change, only decomposition/budget
contract hardening and docs references.
