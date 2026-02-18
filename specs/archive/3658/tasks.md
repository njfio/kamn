# Issue #3658 Tasks

- Issue: `#3658`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing shell contract expectations for observability TLS markers in runtime/local-heavy lanes.
- T2 (Green): implement observability endpoint TLS serving and HTTPS integration selector coverage.
- T3 (Green): propagate TLS marker outputs through runtime/local-heavy lane reports and policies.
- T4 (Regression): add marker-drift tamper checks for observability TLS route markers.
- T5 (Docs): update CI strategy observability lane section for TLS local-heavy markers and route coverage.
- T6 (Verify): run:
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes -- --exact --nocapture`
  - `bash scripts/runtime/test_validate_runtime_observability_endpoint_live.sh`
  - `bash scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
  - `bash scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
  - `bash scripts/runtime/test_validate_local_observability_scrape_live.sh`
  - `bash scripts/runtime/test_check_local_observability_scrape_live_policy.sh`
  - `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`

## Completion Evidence
- HTTPS observability route selectors pass and local-heavy lane policy/contracts enforce deterministic TLS route markers fail-closed.
