# Tasks: Issue #4433

Status: Completed
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
  - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
    - Failed with: `expected compose topology contract lane packaging reason taxonomy marker`
  - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
    - Failed with: `expected compose topology policy checker reason taxonomy marker`
  - `bash scripts/deploy/test_validate_deployment_assets_live.sh`
    - Failed with: `expected deployment compose-manifest contract marker`

- GREEN command/output:
  - `bash scripts/deploy/test_validate_deployment_assets_live.sh`
    - Passed: `deployment assets live validation tests passed.`
  - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
    - Passed: `compose topology contract lane tests passed.`
  - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
    - Passed: `compose topology policy tests passed.`
  - `bash scripts/deploy/test_deployment_assets.sh`
    - Passed: `deployment asset contract tests passed.`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed

- Regression summary:
  - Compose topology lane now emits deterministic packaging taxonomy and evidence markers.
  - Policy checker now fails closed on packaging taxonomy/reason/evidence drift.
  - Live deployment asset validation now includes compose-manifest-config marker coverage.
