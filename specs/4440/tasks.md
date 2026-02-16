# Tasks: Issue #4440

Status: In Progress
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
  - Pending from #4439.

- GREEN command/output:
  - Pending implementation.
