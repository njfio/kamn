# Tasks - Issue #3813

- [x] T1 (Red): define deterministic drift/failure scenarios for this issue scope.
- [x] T2 (Green): deliver stable contract behavior for mapped shutdown/signal suites.
- [x] T3 (Refactor/Docs): preserve marker and governance traceability.
- [x] T4 (Verify): run mapped conformance suites.

## Planned Verification Commands

- 'bash scripts/ci/test_run_daemon_os_signal_reproducer.sh'
- 'bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh'
- 'bash scripts/runtime/test_validate_daemon_os_signal_live.sh'
- 'bash scripts/runtime/test_check_local_signal_secret_hygiene_live_policy.sh'
- 'bash scripts/runtime/test_validate_local_signal_secret_hygiene_live_contract_lane.sh'
- 'bash scripts/ci/test_local_signal_secret_hygiene_ci_exclusion_policy.sh'
- 'bash scripts/runtime/test_check_service_api_graceful_shutdown_drain_live_policy.sh'
- 'bash scripts/runtime/test_validate_service_api_graceful_shutdown_drain_live_contract_lane.sh'
- 'bash scripts/ci/test_service_api_graceful_shutdown_drain_ci_exclusion_policy.sh'
- 'bash scripts/runtime/test_check_service_api_shutdown_abrupt_close_regression_live_policy.sh'
- 'bash scripts/runtime/test_validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh'
- 'bash scripts/ci/test_service_api_shutdown_abrupt_close_regression_ci_exclusion_policy.sh'
- 'bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh'
- 'bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh'
- 'bash scripts/runtime/test_run_lifecycle_property_contract_lane.sh'
