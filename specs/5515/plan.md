# Issue #5515 Plan - Telemetry Spec Status Normalization

## Approach
1. Add RED lifecycle docs-contract assertions for `specs/5513/spec.md` status.
2. Update `specs/5513/spec.md` status to `Implemented`.
3. Run targeted lifecycle and regression docs-contract tests.

## Affected Modules
- `specs/5513/spec.md`
- `crates/kamn-core/tests/review_r50_spec_status_lifecycle_docs_contract.rs`
- `specs/milestones/r50-23-spec-status-normalization-merged-telemetry-task/index.md`
- `specs/5515/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: future status drift on merged specs.
  - Mitigation: expand lifecycle contract checks to include 5513.

## Interfaces / Contracts
- Spec lifecycle status metadata contract only.

## Validation Strategy
- `cargo test -p kamn-core --test review_r50_spec_status_lifecycle_docs_contract -- --nocapture`
- `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts -- --exact`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
