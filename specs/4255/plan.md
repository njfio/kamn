# Plan — #4255 Partition Mismatch Red Tests

Status: Reviewed

## Approach

1. Extend `test_check_block_reconciliation_partition_rejoin_live_policy.sh` with:
   - missing marker fixture
   - unsorted/duplicated reason-code fixture
   - repeated deterministic mismatch assertion
2. Keep fixtures minimal and deterministic to avoid flake.

## Risks

- Overlapping failure reasons may hide assertions.
  - Mitigation: isolate one mutated marker per fixture and assert expected reason code presence.
