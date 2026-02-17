# Plan — #4388

Status: Implemented

## Approach

- Extend live transport fault-matrix tests with RED assertions for peer reason markers and deterministic tamper rejection.
- Update live transport policy checker to validate peer-integrity and retry-timeout markers with deterministic mismatch reasons.
- Add docs-contract parity checks for peer reason matrix markers in release checklist and Kolme devnet ops docs.
- Wire new peer reason markers through contract-lane report/output and tests.
- Update docs to publish required peer reason matrix markers.

## Affected Areas

- `scripts/runtime/live_transport_fault_matrix_live_contract.py`
- `scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh`
- `scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/planning/kolme-devnet-ops.md`

## Risks and Mitigations

- Risk: stricter peer reason checks may reject previously accepted reports.
  - Mitigation: add deterministic reason-code mapping and update tests/docs in the same change set.
- Risk: docs-parity checks can be noisy if marker strings drift.
  - Mitigation: enforce exact marker strings and co-locate docs updates in this PR.

## Interfaces / Contracts

- Validation output additions:
  - `peer_adapter_reason_taxonomy_version`
  - `peer_integrity_fail_closed_reason_code`
  - `peer_adapter_reason_projection_timeout_code`
  - `peer_adapter_reason_projection_budget_exhausted_code`
  - `peer_adapter_multi_process_validation_local_heavy_status`
- Policy output additions:
  - deterministic peer reason marker fields above.
