# Plan — #4405

Status: Reviewed

## Approach

- Extend `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh` with red cases that require deterministic invariant reason-taxonomy and expected/observed reason mapping markers.
- Add tampered fixtures that intentionally encode lane-status acceptance drift and taxonomy drift.
- Keep tests focused on policy checker behavior to avoid broad runtime changes.

## Affected Areas

- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`

## Risks and Mitigations

- Risk: red tests fail due missing implementation (expected).
  - Mitigation: immediately follow with #4406 implementation to restore green.

## Contract Notes

- Preserve existing tamper regression assertions (artifact-key mismatch) while adding new red coverage.
