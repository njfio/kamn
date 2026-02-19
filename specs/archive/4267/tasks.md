# Tasks — #4267 Protocol Taxonomy + Runbook Marker Parity

Status: Implemented

## Ordered Tasks

- T1 (Regression/Functional): add red checks for taxonomy drift and runbook marker divergence in `test_validate_service_api_axum_ingress_live_contract_lane.sh`.
- T2 (Implementation/Integration): add optional runbook parity enforcement hook in `service_api_contract_lane_runner.sh`.
- T3 (Implementation/Integration): configure axum ingress lane runbook parity marker contract in `validate_service_api_axum_ingress_live_contract_lane.sh`.
- T4 (Conformance/Docs): add runbook/checklist marker sections and docs-contract assertions.
- T5 (Verification): run lane + docs tests and targeted rust docs tests.
