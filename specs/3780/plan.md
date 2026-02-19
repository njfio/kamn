# Issue #3780 Plan

- Issue: #3780
- Status: In Progress
- Spec: `specs/3780/spec.md`

## Approach
1. Record parent-level lineage for completed child implementations (`#3794`, `#3795`).
2. Verify parent ACs by running focused lane/policy/exclusion/docs contract checks.
3. Close parent issue with checklist/label/closure marker updates once verification is green.

## Child Integration Summary
- `#3794` delivered deterministic `retry_reconnect_marker_contract_status=verified` propagation across:
  - run-lane summary output,
  - policy checker output/report,
  - contract-lane aggregated report/output,
  - Kolme devnet docs marker surface.
- `#3795` delivered CI-fast exclusion/docs parity tightening across:
  - selector-gated local-heavy workflow assertion,
  - fast-mode run-command leakage checks,
  - strategy docs reason-marker taxonomy parity assertions.

## Risks and Mitigations
- Risk: Parent task closure without explicit lineage artifacts.
  - Mitigation: add `specs/3780/*` and update parent child checklist.
- Risk: drift between child lane/policy markers and docs exclusion contracts.
  - Mitigation: run focused combined verification bundle before close.

## Verification Bundle
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `bash scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`
- `bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`
- `cargo test -p kamn-node --test kolme_devnet_ops_docs`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_live_transport_fault_matrix_ci_exclusion_policy_contract_markers -- --exact`
