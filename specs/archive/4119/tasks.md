# Tasks - Issue #4119

- [x] T1 (Red): define marker-lineage and local-heavy exclusion failure scenarios.
- [x] T2 (Green): deliver deterministic CI checker and exclusion-policy behavior.
- [x] T3 (Refactor/Docs): keep CI strategy and closure markers aligned to checker taxonomy.
- [x] T4 (Verify): run checker, exclusion-policy, and integration contract-lane suites.

## Planned Verification Commands

- `bash scripts/ci/test_check_observability_endpoint_drift_contract.sh`
- `bash scripts/ci/test_local_observability_scrape_ci_exclusion_policy.sh`
- `bash scripts/ci/test_service_api_prometheus_metrics_ci_exclusion_policy.sh`
- `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`
- `bash scripts/runtime/test_validate_service_api_prometheus_metrics_live_contract_lane.sh`
