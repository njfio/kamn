# Tasks - Issue #4115

- [x] T1 (Red): define deterministic telemetry/health and governance drift failure scenarios.
- [x] T2 (Green): deliver projection/emission parity plus checker/policy coverage via child tasks.
- [x] T3 (Refactor/Docs): keep CI/deploy/planning observability references synchronized.
- [x] T4 (Verify): run runtime and CI observability conformance suites.

## Planned Verification Commands

- `bash scripts/runtime/test_check_local_observability_scrape_live_policy.sh`
- `bash scripts/runtime/test_check_service_api_prometheus_metrics_live_policy.sh`
- `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`
- `bash scripts/runtime/test_validate_service_api_prometheus_metrics_live_contract_lane.sh`
- `bash scripts/ci/test_check_observability_endpoint_drift_contract.sh`
- `bash scripts/ci/test_local_observability_scrape_ci_exclusion_policy.sh`
- `bash scripts/ci/test_service_api_prometheus_metrics_ci_exclusion_policy.sh`
