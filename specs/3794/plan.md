# Issue #3794 Plan

- Issue: #3794
- Status: In Progress
- Spec: `specs/3794/spec.md`

## Implementation Approach
1. Introduce a deterministic retry/reconnect marker contract status field in the transport resilience lane run report and policy report.
2. Require and validate the field in policy checks so drift fails closed.
3. Propagate the marker through the contract-lane summary report and emitted markers.
4. Update transport resilience test harnesses and docs marker references.

## Affected Modules
- `scripts/runtime/live_transport_fault_matrix_live_contract.py`
- `scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh`
- `scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`
- `docs/planning/kolme-devnet-ops.md`

## Risks and Mitigations
- Risk: marker propagation drift between run-lane, policy, and contract-lane reports.
  - Mitigation: enforce marker presence and value in all three existing contract test suites.
- Risk: shell-surface growth.
  - Mitigation: keep changes minimal and focused on existing scripts/tests; run shell LOC/ratio/ratchet guardrails before PR.

## Contracts and Interfaces
- Run-lane report contract (`kamn.runtime.live-transport-fault-matrix-report.v1`) includes:
  - `retry_reconnect_marker_contract_status=verified`
- Policy report contract (`kamn.runtime.live-transport-fault-matrix-policy-report.v1`) includes:
  - `retry_reconnect_marker_contract_status=verified`
- Contract-lane report contract (`kamn.runtime.live-transport-fault-matrix-contract-lane-report.v1`) propagates:
  - `retry_reconnect_marker_contract_status=verified`

## Verification Strategy
- Execute RED by adding marker assertions before implementation.
- Execute GREEN by adding marker emission/validation/propagation.
- Execute regression lane and shell guardrail checks listed in `specs/3794/spec.md`.
