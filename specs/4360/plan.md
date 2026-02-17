# Plan: #4360 Red Tests

## Approach

1. Extend `scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh` with two new synthetic mutation checks:
   - remove key-source contract-version marker;
   - remove runtime command key-source marker.
2. Assert deterministic reason codes through checker stderr output.
3. Keep existing fallback leakage check as regression evidence.

## Risks

- Test fixture mutation may accidentally invalidate unrelated marker expectations.
  - Mitigation: isolate each mutation to one field and assert only targeted reason.
