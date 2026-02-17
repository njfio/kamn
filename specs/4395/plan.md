# Plan — #4395

Status: Implemented

## Approach

- Add peer reason marker fields + checks to `live_transport_fault_matrix_live_contract.py` run-lane/policy paths.
- Add deterministic mismatch reason mapping for peer reason marker drift.
- Add docs parity checks for peer marker matrix in release checklist + Kolme devnet ops.
- Propagate peer marker fields through contract-lane output/report and tests.

## Affected Areas

- `scripts/runtime/live_transport_fault_matrix_live_contract.py`
- `scripts/runtime/validate_live_transport_fault_matrix_live_contract_lane.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/planning/kolme-devnet-ops.md`

## Risks and Mitigations

- Risk: legacy reports missing new marker fields fail policy.
  - Mitigation: deterministic required-field mismatch reasons and coordinated test/docs updates.
- Risk: docs parity introduces maintenance coupling.
  - Mitigation: enforce concise deterministic marker strings and keep docs update in lockstep.
