# Issue #5511 Plan - Spec Status Lifecycle Contract Normalization

## Approach
1. Add RED docs-contract test asserting implemented status for targeted spec files.
2. Update targeted spec status lines from `Accepted` to `Implemented`.
3. Run targeted docs-contract tests and format checks.

## Affected Modules
- `specs/5507/spec.md`
- `specs/5509/spec.md`
- `crates/kamn-core/tests/review_r50_spec_status_lifecycle_docs_contract.rs`
- `specs/milestones/r50-21-spec-status-lifecycle-normalization/index.md`
- `specs/5511/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: future lifecycle drift on closed specs.
  - Mitigation: docs-contract test pins required status lines.

## Interfaces / Contracts
- Spec artifact lifecycle contract only.

## Validation Strategy
- `cargo test -p kamn-core --test review_r50_spec_status_lifecycle_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r50_governance_feature_rebalancing_docs_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
