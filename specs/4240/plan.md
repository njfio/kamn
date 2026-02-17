# Plan — #4240 Red Tests for Append/Checkpoint Mismatch

Status: Reviewed

## Approach

1. Extend `scripts/runtime/test_check_sqlite_crash_recovery_live_policy.sh` fixture inventory.
2. Add explicit tampered WAL append status fixture.
3. Reuse checkpoint drift fixture to assert parity mismatch reason marker once checker mapping is added.
4. Keep baseline path and existing tamper coverage unchanged.

## Risks

- Fixture overlap with existing checkpoint drift checks may cause ambiguous assertions.
  Mitigation: assert required reason marker presence for each fixture.
