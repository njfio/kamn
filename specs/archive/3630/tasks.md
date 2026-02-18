# Issue #3630 Tasks

- Issue: `#3630`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add missing TLS policy/contract-lane assertions for service + observability.
- T2 (Green): wire TLS marker generation and checks into runtime/deploy lanes.
- T3 (Regression): add docs/governance checks for TLS go/no-go evidence.
- T4 (Verify): run
  - `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
  - `bash scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
  - `bash scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
  - `bash scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`

## Completion Evidence
- TLS service/observability/governance contract lanes pass with deterministic evidence markers.
