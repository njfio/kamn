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
- `runtime_phase_parity_reason_codes_value=<normalized runtime extraction reason key>`
- `runtime_phase_parity_reason_mapper=runtime_phase_parity_reason_mapper_v1`
- `runtime_phase_parity_evidence_outputs_csv=runtime_phase_module_parity_status,runtime_extraction_evidence_output_status,ci_local_runtime_phase_parity_budget_boundary_status`

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

## Runtime Module Boundary Parity Drift Cases (Issue #4329)

Issue lineage:
- Task: `#4329`
- Subtasks: `#4336`, `#4337`

Deterministic runtime module-boundary parity markers:
- `runtime_module_boundary_parity_reason_taxonomy_version=kamn.runtime.module-boundary-parity-reason-taxonomy.v1`
- `runtime_module_boundary_parity_reason_codes_csv=runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded`
- `runtime_module_boundary_reason_codes_value=<normalized runtime module-boundary reason key>`
- `runtime_module_boundary_evidence_outputs_csv=runtime_module_boundary_parity_status,runtime_module_boundary_evidence_status,ci_local_runtime_module_boundary_budget_boundary_status`
- `runtime_module_boundary_parity_status=verified`
- `runtime_module_boundary_evidence_status=verified`
- `ci_local_runtime_module_boundary_budget_boundary_status=verified`

Deterministic fail-closed boundary drift reasons:
- `runtime_orchestration_dispatch_boundary_drift_detected`
- `runtime_daemon_phase_boundary_drift_detected`
- `runtime_kolme_live_boundary_drift_detected`
- `ci_local_runtime_module_boundary_budget_boundary_exceeded`

Boundary parity guard commands:
- `cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test runtime_architecture_docs -- --nocapture`

## Docs Governance and Rustdoc Navigation Parity

Issue lineage:
- Task: `#4524`
- Subtasks: `#4531`, `#4532`

Docs graduation velocity governance markers:
- `reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-velocity-reason-taxonomy.v1`
- `reason_codes_csv=allowlist_fully_graduated,baseline_window_not_elapsed,ci_local_docs_velocity_window_boundary_exceeded,multiple_policy_violations,stagnation_window_exceeded,velocity_target_met,velocity_threshold_config_invalid,velocity_window_under_threshold,window_not_elapsed`
- `reason_codes_value=<deterministic reason key>`

CI-local docs velocity boundary:
- policy validation fails closed when `velocity_window_commits > 240`.
- boundary marker: `reason_codes_value=ci_local_docs_velocity_window_boundary_exceeded`.

Rustdoc navigation parity drift markers:
- `reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-policy-reason-taxonomy.v1`
- `reason_codes_csv=graduated_module_exemption_regression,rustdoc_navigation_parity_drift`
- `reason_code=rustdoc_navigation_parity_drift`

Docs governance entrypoints:
- `bash scripts/ci/test_missing_docs_velocity_guard_contract.sh`
- `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`

Missing-docs graduation evidence marker contract (`check_kamn_core_missing_docs_policy.sh`):
- `missing_docs_allowlisted_module_count=<int>`
- `missing_docs_graduated_module_count=<int>`
- `missing_docs_allowlisted_module_delta=<int>`
- `missing_docs_graduated_module_delta=<int>`
- `missing_docs_velocity_status=pass|fail`
- `missing_docs_velocity_final_decision=GO|HOLD`
- `missing_docs_velocity_reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-velocity-reason-taxonomy.v1`
- `missing_docs_velocity_reason_codes_csv=allowlist_fully_graduated,baseline_window_not_elapsed,ci_local_docs_velocity_window_boundary_exceeded,multiple_policy_violations,stagnation_window_exceeded,velocity_target_met,velocity_threshold_config_invalid,velocity_window_under_threshold,window_not_elapsed`
- `missing_docs_velocity_reason_codes_value=<deterministic reason key>`

Rustdoc navigation publication ratio-governance marker contract (`check_kamn_core_rustdoc_artifact_policy.sh`):
- `rustdoc_navigation_ratio_status=within|exceeded`
- `runtime_budget_status=within|exceeded`
- `docs_contract_test_count=<int>`
- `behavioral_test_count=<int>`
- `docs_contract_to_behavioral_ratio=<float>`
- `max_docs_contract_to_behavioral_ratio=<float>`
- `reason_taxonomy_version=kamn.ci.kamn-core-rustdoc-navigation-governance-reason-taxonomy.v1`
- `reason_codes_csv=docs_behavioral_ratio_threshold_exceeded,rustdoc_artifact_runtime_budget_exceeded,rustdoc_artifact_policy_validation_failed`
- `reason_code=docs_behavioral_ratio_threshold_exceeded`
