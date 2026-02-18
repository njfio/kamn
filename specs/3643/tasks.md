# Issue #3643 Tasks

- Issue: `#3643`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add missing observability TLS policy and contract checks.
- T2 (Green): wire TLS compatibility/fail-closed markers for observability routes.
- T3 (Regression): run transport-observability TLS CI smoke convergence checks.
- T4 (Verify): run
  - `bash scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
  - `bash scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
  - `bash scripts/ci/test_check_transport_observability_tls_ci_smoke_convergence.sh`

## Completion Evidence
- Observability TLS compatibility and fail-closed checks are green.
