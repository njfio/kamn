# Issue #5507 Plan - R50 Doc-Contract Consolidation Guardrail Contractization

## Approach
1. Add a new docs-contract test asserting required consolidation markers and arithmetic invariants.
2. Update `docs/review/gaps-and-issues-r50.md` with deterministic consolidation marker lines and status wording.
3. Run targeted docs-contract tests plus formatting checks.

## Affected Modules
- `docs/review/gaps-and-issues-r50.md`
- `crates/kamn-core/tests/review_r50_doc_contract_consolidation_docs_contract.rs`
- `specs/milestones/r50-19-doc-contract-suite-consolidation-guardrail-contracts/index.md`
- `specs/5507/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker arithmetic or key drift breaks contracts.
  - Mitigation: parse numeric marker values and assert derived relationships in tests.

## Interfaces / Contracts
- Review-artifact marker contract only (`key=value` lines in R50 report).

## Validation Strategy
- `cargo test -p kamn-core --test review_r50_doc_contract_consolidation_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r50_governance_loop_mitigation_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
