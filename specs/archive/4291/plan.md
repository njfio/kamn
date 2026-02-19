# Plan — #4291

Status: Reviewed

## Approach

- Extend existing CI smoke checker script/test harness for failover markers.
- Add RED tests for:
  - marker drift rejection
  - fast-gate heavy-lane exclusion enforcement
  - deterministic repeated mismatch ordering
- Implement deterministic reason-code mapping and fail-closed behavior.

## Affected Areas

- `scripts/ci/*` failover smoke checker implementation
- `scripts/ci/test_ci_tools.sh` (if checker is composed there)
- checker-specific shell tests

## Risks and Mitigations

- Risk: CI false positives from unstable marker parsing.
  - Mitigation: deterministic normalization and repeated-run ordering tests.
