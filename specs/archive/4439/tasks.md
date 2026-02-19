# Tasks: Issue #4439

Status: Completed
Issue: #4439

## Ordered Tasks

T1 (RED):
- Update `scripts/deploy/test_validate_compose_topology_contract_lane.sh` to assert packaging
  taxonomy/evidence markers and deterministic drift failures.
- Update `scripts/deploy/test_check_compose_topology_contract_policy.sh` to assert deterministic
  taxonomy/reason CSV mismatch handling.

T2 (Verify RED):
- Run:
  - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
  - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
- Confirm failures are RED and deterministic before implementation changes.

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
