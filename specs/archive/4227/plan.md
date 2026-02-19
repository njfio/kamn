# Plan — #4227 Red Tests for Admission Taxonomy/Runbook Divergence

Status: Implemented

## Approach
- Extend `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh` with admission decision taxonomy red/tamper cases.
- Extend `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh` with runbook drift/divergence fixtures for admission decision markers.
- Ensure deterministic reason-code assertions and no flaky timing dependencies.

## Risks
- Drift in reason-code literals causing brittle tests.
  - Mitigation: assert exact deterministic literals and reuse canonical marker constants.
