# Issue #4379 Plan

Status: Reviewed

## Approach

1. Extend `scripts/kolme/test_check_local_signed_to_kolme_demo_policy.sh` with deterministic RED mutations for simulated signing and native marker absence.
2. Extend `scripts/kolme/test_run_local_signed_to_kolme_demo_contract_lane.sh` to assert native signer taxonomy keys in policy reports.
3. Keep existing regression assertions untouched.

## Risks

- Test fixture drift vs evolving summary schema.
  - Mitigation: mutate only stable fields/commands already used in current checks.
