# Plan — #4197 Local Full-Stack Harness Taxonomy Drift and Runbook Divergence Red Tests

## Approach
1. Extend `scripts/runtime/test_check_local_full_stack_integration_live_policy.sh` with additional taxonomy drift tamper assertions for runtime-phase and runtime-module parity markers.
2. Extend `scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh` with runbook parity checks against `docs/deploy/kolme_devnet_ops.md` plus tampered-runbook divergence assertions.
3. Add a deploy runbook section in `docs/deploy/kolme_devnet_ops.md` for local full-stack harness taxonomy and runbook marker mapping.
4. Add docs-contract assertions in `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`.

## Affected Modules
- `scripts/runtime/test_check_local_full_stack_integration_live_policy.sh`
- `scripts/runtime/test_validate_local_full_stack_integration_live_contract_lane.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Risks / Mitigations
- Risk: very long marker strings can introduce brittle copy mistakes.
- Mitigation: centralize expected marker constants in tests and assert exact deterministic values.

- Risk: runbook drift check logic could be non-deterministic.
- Mitigation: enforce ordered required marker list and deterministic reason code mapping in the test path.

## Interfaces / Contracts
- Local full-stack policy mismatch reasons:
  - `local_full_stack_integration_policy_runtime_phase_parity_reason_taxonomy_version_mismatch`
  - `local_full_stack_integration_policy_runtime_phase_parity_reason_codes_csv_mismatch`
  - `local_full_stack_integration_policy_runtime_module_boundary_parity_reason_codes_csv_mismatch`
- Runbook divergence reason markers introduced for red tests:
  - `local_full_stack_harness_taxonomy_mapping_drift_detected`
  - `runbook_marker_parity_mismatch`
