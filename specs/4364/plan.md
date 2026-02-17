# Plan: #4364 RED Tests

## Approach

1. Extend `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`:
   - assert taxonomy fields in valid GO output;
   - assert taxonomy observed value includes targeted NO-GO reasons.
2. Execute script to capture failing-first behavior.
