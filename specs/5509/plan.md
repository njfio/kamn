# Issue #5509 Plan - Governance-Feature Activity Rebalancing Contractization

## Approach
1. Add RED docs-contract tests for required governance-feature rebalancing markers and formulas.
2. Update `docs/review/gaps-and-issues-r50.md` with deterministic rebalancing marker lines and active-status wording.
3. Run targeted docs-contract suites and format checks.

## Affected Modules
- `docs/review/gaps-and-issues-r50.md`
- `crates/kamn-core/tests/review_r50_governance_feature_rebalancing_docs_contract.rs`
- `specs/milestones/r50-20-governance-feature-activity-rebalancing-contracts/index.md`
- `specs/5509/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: mismatch between ratio targets and integer commit targets.
  - Mitigation: explicit integer + float marker parsing with deterministic assertions.

## Interfaces / Contracts
- Review artifact marker contract (`key=value` lines) only.

## Validation Strategy
- `cargo test -p kamn-core --test review_r50_governance_feature_rebalancing_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r50_doc_contract_consolidation_docs_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
