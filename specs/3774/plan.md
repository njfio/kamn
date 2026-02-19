# Issue #3774 Plan

- Issue: #3774
- Status: In Progress
- Spec: `specs/3774/spec.md`

## Approach
1. Capture story-level lineage for completed child task deliveries (`#3778`, `#3779`, `#3780`).
2. Run a focused story verification bundle spanning reconnect/taxonomy + transport resilience exclusion contracts.
3. Close story with checklist/labels/closure markers once the bundle is green.

## Child Integration Summary
- `#3778`: deterministic shared HTTP retry/backoff hardening delivered and closed.
- `#3779`: notifications reconnect pacing + terminal taxonomy hardening delivered and closed.
- `#3780`: local-heavy transport resilience lane and CI exclusion/docs parity hardening delivered and closed.

## Risks and Mitigations
- Risk: story remains open despite complete child implementation.
  - Mitigation: update child checklist and add story-level closure artifacts.
- Risk: cross-task retry/reconnect/transport lane contracts drift after child merges.
  - Mitigation: run combined verification across child contract surfaces before story close.

## Verification Bundle
- `cargo test -p kamn-kolme --test notification_policy_contracts`
- `cargo test -p kamn-core --test kolme_runtime_commit_notifications`
- `cargo test -p kamn-node --test kolme_runtime_commit_docs`
- `cargo test -p kamn-core --test runtime_network_docs`
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `bash scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `bash scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`
- `bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_live_transport_fault_matrix_ci_exclusion_policy_contract_markers -- --exact`
