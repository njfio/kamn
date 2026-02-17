# Plan: #4362 RED Tests

## Approach

1. Extend `scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`:
   - assert signature-decision taxonomy fields in GO checks;
   - assert mapped signature-decision reason values in quorum drift failures.
2. Execute script to capture RED before implementation.
