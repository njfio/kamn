# Runtime Architecture Contract

This document captures runtime extraction and fallback contracts used by
`kamn-node` and the local-heavy validation lanes.

## Runtime Extraction Fallback Taxonomy

Issue lineage:
- Task: `#4537`
- Subtasks: `#4542`, `#4543`

Deterministic taxonomy markers for local full-runtime evidence:
- `runtime_error_reason_taxonomy_version=kamn.runtime.local-full-runtime-error-reason-taxonomy.v1`
- `runtime_error_reason_codes_csv=runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded`

Deterministic fallback gate markers:
- `runtime_shutdown_gate_status=verified`
- `runtime_fallback_classification_status=verified`
- `ci_local_runtime_extraction_budget_boundary_status=verified`

Deterministic fail-closed reasons:
- `runtime_full_shutdown_gate_drift_detected`
- `runtime_fallback_classification_unstable`
- `ci_local_runtime_extraction_budget_boundary_exceeded`

## Evidence and Policy Entrypoints

- `bash scripts/runtime/validate_local_full_runtime_live.sh --mode dry-run --output-json /tmp/local-full-runtime-live-summary.json`
- `bash scripts/runtime/check_local_full_runtime_live_policy.sh --report-file /tmp/local-full-runtime-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-full-runtime-live-policy.json`
- `bash scripts/runtime/validate_local_full_runtime_live_contract_lane.sh --output-json /tmp/local-full-runtime-live-contract-lane-report.json --policy-output-json /tmp/local-full-runtime-live-policy.json`

The contract lane enforces a ci-local extraction boundary of `240` seconds and
fails closed when `--max-seconds` exceeds that boundary.

## Runtime Phase Extraction Parity Taxonomy

Issue lineage:
- Task: `#4536`
- Subtasks: `#4540`, `#4541`

Deterministic taxonomy markers for local full-stack extraction parity evidence:
- `runtime_phase_parity_reason_taxonomy_version=kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1`
- `runtime_phase_parity_reason_codes_csv=runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded`

Deterministic phase parity governance markers:
- `runtime_phase_module_parity_status=verified`
- `runtime_extraction_evidence_output_status=verified`
- `ci_local_runtime_phase_parity_budget_boundary_status=verified`

Deterministic fail-closed reasons:
- `runtime_phase_module_parity_drift_detected`
- `runtime_extraction_evidence_output_unstable`
- `ci_local_runtime_phase_parity_budget_boundary_exceeded`

Phase parity entrypoints:
- `bash scripts/runtime/validate_local_full_stack_integration_live.sh --mode dry-run --max-seconds 240 --output-json /tmp/local-full-stack-integration-summary.json`
- `bash scripts/runtime/check_local_full_stack_integration_live_policy.sh --report-file /tmp/local-full-stack-integration-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-full-stack-integration-policy.json`
- `bash scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh --mode dry-run --max-seconds 240 --output-json /tmp/local-full-stack-integration-contract-lane-report.json --policy-output-json /tmp/local-full-stack-integration-policy.json`
