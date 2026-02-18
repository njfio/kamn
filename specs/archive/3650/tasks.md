# Issue #3650 Tasks

- Issue: `#3650`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing assertions for compatibility and governance markers.
- T2 (Green): implement/update unified compatibility and parity policy checks.
- T3 (Regression): enforce CI exclusion policy for local-heavy validations.
- T4 (Verify): run
  - `bash scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
  - `bash scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
  - `bash scripts/runtime/test_check_service_api_reason_code_compatibility_live_policy.sh`
  - `bash scripts/runtime/test_check_service_api_serde_payload_parity_live_policy.sh`
  - `bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh`

## Completion Evidence
- Unified stack compatibility matrix and governance checks are green.
