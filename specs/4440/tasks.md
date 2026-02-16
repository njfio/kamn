# Tasks: Issue #4440

Status: Completed
Issue: #4440

## Ordered Tasks

T1 (Input from #4439 RED):
- Consume RED failures proving missing taxonomy/evidence marker contract.

T2 (GREEN, Implementation):
- Implement packaging taxonomy/evidence marker output in lane summary and stdout.
- Implement deterministic policy mismatch reasons for taxonomy/evidence drift.

T3 (GREEN, Docs):
- Add packaging taxonomy marker references to deployment + CI strategy docs.

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

- GREEN command/output:
  - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
    - Passed: `compose topology contract lane tests passed.`
  - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
    - Passed: `compose topology policy tests passed.`
  - `bash scripts/deploy/test_validate_deployment_assets_live.sh`
    - Passed: `deployment assets live validation tests passed.`
