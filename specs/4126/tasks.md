# Tasks - Issue #4126

- [x] T1 (Red): define marker-lineage and CI exclusion failure scenarios.
- [x] T2 (Green): deliver deterministic checker/exclusion policy behavior.
- [x] T3 (Refactor/Docs): preserve CI boundary and marker taxonomy contracts.
- [x] T4 (Verify): execute checker and exclusion-policy suites.

## Planned Verification Commands

- `bash scripts/ci/test_check_observability_endpoint_drift_contract.sh`
- `bash scripts/ci/test_local_observability_scrape_ci_exclusion_policy.sh`
- `bash scripts/ci/test_service_api_prometheus_metrics_ci_exclusion_policy.sh`
