# Tasks - Issue #4118

- [x] T1 (Red): codify marker-projection failure scenarios for observability metrics and health outputs.
- [x] T2 (Green): wire deterministic observability projection and emission behavior.
- [x] T3 (Refactor/Docs): preserve marker taxonomy references in runtime/deploy guidance.
- [x] T4 (Verify): run projection policy and contract-lane suites.

## Planned Verification Commands

- `bash scripts/runtime/test_check_local_observability_scrape_live_policy.sh`
- `bash scripts/runtime/test_check_service_api_prometheus_metrics_live_policy.sh`
- `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`
- `bash scripts/runtime/test_validate_service_api_prometheus_metrics_live_contract_lane.sh`
