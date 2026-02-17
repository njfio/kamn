# Plan — #4244

Status: Reviewed

## Approach

- Add a dedicated shell test for sqlite crash-replay evidence convergence.
- Generate baseline artifacts via existing contract lane.
- Mutate policy/report artifacts in temp files and assert deterministic fail-closed reasons.

## Affected Areas

- `scripts/runtime/test_check_sqlite_crash_recovery_live_evidence_convergence.sh`
- `scripts/runtime/test_validate_sqlite_crash_recovery_live_contract_lane.sh`

## Risks and Mitigations

- Risk: test brittleness from ad-hoc payload edits.
  - Mitigation: limit mutations to explicit marker fields and assert exact reason codes.
