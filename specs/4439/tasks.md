# Tasks: Issue #4439

Status: In Progress
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
  - Pending execution.

- GREEN command/output:
  - Covered by parent issue #4433 implementation.
