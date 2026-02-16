# Tasks: Issue #4433

Status: In Progress
Issue: #4433

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Extend tests in:
  - `scripts/deploy/test_validate_compose_topology_contract_lane.sh`
  - `scripts/deploy/test_check_compose_topology_contract_policy.sh`
- Add RED assertions for:
  - compose/manifest/config drift rejection,
  - missing packaging taxonomy/evidence markers,
  - deterministic reason-code outputs.

T2 (GREEN, Implementation):
- Implement deterministic packaging taxonomy/evidence fields in:
  - `scripts/deploy/validate_compose_topology_contract_lane.sh`
  - `scripts/deploy/check_compose_topology_contract_policy.sh`
  - supporting marker outputs from `scripts/deploy/validate_deployment_assets_live.sh` as needed.

T3 (GREEN, Docs):
- Update:
  - `docs/ops/deployment.md`
  - `docs/ci/strategy.md`
- Add packaging taxonomy and fail-closed reason marker references.

T4 (Verify):
- Run:
  - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
  - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
  - `bash scripts/deploy/test_validate_deployment_assets_live.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - Pending execution.

- GREEN command/output:
  - Pending implementation.

- Regression summary:
  - Pending verification.
