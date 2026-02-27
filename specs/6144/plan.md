# Plan: Issue #6144

## Approach
1. Capture RED baseline counts for:
   - LOC of heavyweight governance contract test files.
2. Rewrite high-overhead governance test modules to a minimal contract harness:
   - `review_r53_docs_contract.rs`
   - `review_r52_branch_hygiene_reconciliation_docs_contract.rs`
   - `review_r50_doc_contract_consolidation_docs_contract.rs`
3. Keep fail-closed behavior for missing critical markers and preserve deterministic parsing.
4. Run scoped `kamn-core` test verification for the modified modules and quality gates.

## Affected Modules
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_doc_contract_consolidation_docs_contract.rs`
- `specs/6144/spec.md`
- `specs/6144/plan.md`
- `specs/6144/tasks.md`

## Risks / Mitigations
- Risk: Removing markers that other tests still expect causes broad breakage.
  Mitigation: keep the README marker surface intact in this issue and simplify only targeted
  governance contract files with scoped validation.
- Risk: Over-pruning could remove safety-critical checks.
  Mitigation: explicitly keep activity-ratio and freeze/immutability checks as required contracts.
- Risk: Historical review docs may lack newly interpreted keys.
  Mitigation: keep checks keyed to existing stable markers or explicitly validate presence only in
  canonical README contract docs.

## Interfaces / Contracts
- No runtime API or protocol interface changes.
- Governance contract interface changes are limited to static marker definitions and tests.
