# Issue #3632 Tasks

- Issue: `#3632`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing compatibility and marker-drift assertions for unified stack lanes.
- T2 (Green): implement compatibility matrix checks and deterministic marker emission.
- T3 (Regression): enforce CI-fast exclusion governance for local-heavy lanes.
- T4 (Verify): run
  - `bash scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
  - `bash scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
  - `bash scripts/runtime/test_check_service_api_reason_code_compatibility_live_policy.sh`
  - `bash scripts/runtime/test_check_service_api_serde_payload_parity_live_policy.sh`
  - `bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh`

## Completion Evidence
- Unified API-observability compatibility and governance lanes pass with deterministic markers.
