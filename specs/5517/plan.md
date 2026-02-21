# Issue #5517 Plan - 5515 Spec Status Normalization

## Approach
1. Add RED lifecycle docs-contract assertions for `specs/5515/spec.md` status.
2. Update `specs/5515/spec.md` status to `Implemented`.
3. Run targeted lifecycle and regression docs-contract tests.

## Affected Modules
- `specs/5515/spec.md`
- `crates/kamn-core/tests/review_r50_spec_status_lifecycle_docs_contract.rs`
- `specs/milestones/r50-24-spec-status-normalization-merged-issue-5515/index.md`
- `specs/5517/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: repeated lifecycle drift on merged spec artifacts.
  - Mitigation: expand lifecycle contract checks to include `5515`.

## Interfaces / Contracts
- Spec lifecycle status metadata contract only.

## Validation Strategy
- `cargo test -p kamn-core --test review_r50_spec_status_lifecycle_docs_contract -- --nocapture`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
