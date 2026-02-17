# Plan — #4245

Status: Reviewed

## Approach

- Add convergence schema/taxonomy constants to sqlite contract framework.
- Implement `check-evidence-convergence` parity checks between contract-lane report, policy report, and linked source report.
- Reuse deterministic reason-code resolver from policy logic to verify promotion reason mapping.
- Emit convergence report artifact and stable marker output for shell consumers.

## Affected Areas

- `scripts/runtime/sqlite_crash_recovery_live_contract.py`
- `scripts/runtime/check_sqlite_crash_recovery_live_evidence_convergence.sh`
- `scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh`

## Risks and Mitigations

- Risk: duplicated reason mapping logic diverges from policy checker.
  - Mitigation: reuse existing resolver and taxonomy constants directly.
