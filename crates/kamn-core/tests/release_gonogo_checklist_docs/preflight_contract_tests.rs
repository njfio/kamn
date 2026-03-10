use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_PREFLIGHT_GATES_MARKERS: &[&str] = &[
    "## Preflight Gates",
    "Migration plan reviewed and signed",
    "Compatibility matrix validated",
    "CI fast gate and deferred deep lane both green",
    "Rollback runbook version pinned",
];

#[test]
fn checklist_contains_preflight_gates() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_PREFLIGHT_GATES_MARKERS, "checklist_contains_preflight_gates");
}

const CHECKLIST_CONTAINS_STALE_SCRIPT_REFERENCE_DELETION_WAVE_GATE_MARKERS: &[&str] = &[
    "## Stale Script Reference Deletion-Wave Gate (Issues #4960, #4972)",
    "bash scripts/ci/test_check_stale_script_references.sh",
    "bash scripts/ci/check_stale_script_references.sh --output-json /tmp/stale-script-reference-report.json",
    "reason_taxonomy_version=kamn.ci.stale-script-reference-detector-reason-taxonomy.v1",
    "reason_codes_csv=stale_script_reference_argument_invalid,stale_script_reference_deletion_manifest_missing,stale_script_reference_deletion_manifest_schema_invalid,stale_script_reference_detected,stale_script_reference_manifest_entry_invalid,stale_script_reference_output_json_required,stale_script_reference_output_write_failed,stale_script_reference_scan_root_missing",
    "reason_codes=none|<csv>",
    "status=ok|fail",
    "final_decision=GO|NO-GO",
    "stale_reference_count=<n>",
    "stale_script_reference_detected",
    "stale_script_reference_manifest_entry_invalid",
    "stale_script_reference_deletion_manifest_schema_invalid",
    "Regression: #4960",
    "Regression: #4972",
];

#[test]
fn checklist_contains_stale_script_reference_deletion_wave_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_STALE_SCRIPT_REFERENCE_DELETION_WAVE_GATE_MARKERS, "checklist_contains_stale_script_reference_deletion_wave_gate");
}

const CHECKLIST_CONTAINS_PRODUCTION_MODE_LIVE_PROVIDER_ENFORCEMENT_GATE_MARKERS: &[&str] = &[
    "## Production-Mode Live Provider Enforcement Gate (Issue #4371)",
    "test_run_local_kamn_live_runtime_integration_contract_lane.sh",
    "test_run_local_kamn_live_runtime_integration_real_node_profile.sh",
    "runtime_commit_in_memory_provider_reference_detected",
    "runtime_commit_policy_check_in_memory_provider_reference_detected",
    "InMemoryKolmeRuntimeCommitClient",
    "Regression: #4371",
];

#[test]
fn checklist_contains_production_mode_live_provider_enforcement_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_PRODUCTION_MODE_LIVE_PROVIDER_ENFORCEMENT_GATE_MARKERS, "checklist_contains_production_mode_live_provider_enforcement_gate");
}

const CHECKLIST_CONTAINS_FULL_STACK_HARNESS_MARKER_CHECKER_REASON_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Full-Stack Harness Marker Checker Reason Mapping Gate (Issue #4196)",
    "test_check_full_io_scenario_matrix_live_policy.sh",
    "test_validate_full_io_scenario_matrix_live_contract_lane.sh",
    "full_io_harness_policy_reason_taxonomy_version=kamn.runtime.full-io-scenario-matrix-policy-reason-taxonomy.v1",
    "full_io_harness_policy_reason_codes_csv=full_io_scenario_matrix_policy_schema_mismatch,full_io_scenario_matrix_policy_status_mismatch,full_io_scenario_matrix_policy_final_decision_mismatch,full_io_scenario_matrix_policy_ci_fast_gate_mismatch,full_io_scenario_matrix_policy_process_harness_mismatch,full_io_scenario_matrix_policy_api_route_matrix_mismatch,full_io_scenario_matrix_policy_auth_failure_matrix_mismatch,full_io_scenario_matrix_policy_websocket_matrix_mismatch,full_io_scenario_matrix_policy_multinode_propagation_mismatch,full_io_scenario_matrix_policy_fast_gate_exclusion_mismatch,full_io_scenario_matrix_policy_fast_gate_reason_mismatch,full_io_scenario_matrix_policy_lane_mode_invalid,full_io_scenario_matrix_policy_command_count_invalid,full_io_scenario_matrix_policy_artifact_paths_invalid,full_io_scenario_matrix_policy_dry_run_eligibility_mismatch,full_io_scenario_matrix_policy_dry_run_command_count_mismatch,full_io_scenario_matrix_policy_dry_run_command_status_mismatch,full_io_scenario_matrix_policy_dry_run_reason_code_mismatch,full_io_scenario_matrix_policy_run_mode_exclusion_mismatch,full_io_scenario_matrix_policy_run_mode_command_count_mismatch,full_io_scenario_matrix_policy_run_mode_command_status_mismatch,full_io_scenario_matrix_policy_run_mode_reason_code_mismatch,full_io_scenario_matrix_policy_expected_decision_mismatch",
    "full_io_harness_policy_reason_codes_value=none|<csv>",
    "full_io_scenario_matrix_policy_status=verified|failed",
    "full_io_scenario_matrix_policy_process_harness_mismatch",
    "full_io_scenario_matrix_policy_dry_run_command_count_mismatch",
    "full_io_scenario_matrix_policy_dry_run_command_status_mismatch",
    "full_io_scenario_matrix_policy_expected_decision_mismatch",
    "Regression: #4196",
];

#[test]
fn checklist_contains_full_stack_harness_marker_checker_reason_mapping_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_FULL_STACK_HARNESS_MARKER_CHECKER_REASON_MAPPING_GATE_MARKERS, "checklist_contains_full_stack_harness_marker_checker_reason_mapping_gate");
}

const CHECKLIST_CONTAINS_RUNTIME_SIGNER_KEY_SOURCE_REASON_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Runtime Signer Key-Source/Fallback Reason Mapping Gate (Issue #4356)",
    "key_source_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-key-source-reason-taxonomy.v1",
    "key_source_reason_codes_csv=runtime_signer_key_source_contract_version_missing,runtime_signer_key_source_contract_version_mismatch,runtime_signer_key_source_contract_version_contract_mismatch,runtime_signer_key_source_missing,runtime_signer_key_source_invalid,runtime_signer_key_source_profile_pair_disallowed,runtime_signer_key_source_contract_mismatch,runtime_commit_signer_key_source_marker_missing,runtime_commit_fallback_private_key_command_marker_detected,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation",
    "key_source_reason_codes_value=none|<csv>",
    "runtime_commit_signer_key_source_marker_missing",
    "Regression: #4356",
];

#[test]
fn checklist_contains_runtime_signer_key_source_reason_mapping_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_RUNTIME_SIGNER_KEY_SOURCE_REASON_MAPPING_GATE_MARKERS, "checklist_contains_runtime_signer_key_source_reason_mapping_gate");
}

const CHECKLIST_CONTAINS_ROTATION_PREFLIGHT_QUORUM_PARITY_AND_CUSTODY_REASON_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Rotation Preflight Quorum Marker Parity and Custody Reason Mapping Gate (Issues #4169, #4170)",
    "test_check_local_kolme_live_deployment_preflight_policy.sh",
    "check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-deployment-preflight-policy.json",
    "rotation_preflight_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1",
    "rotation_preflight_reason_codes_csv=signer_key_source_contract_version_mismatch,signer_key_source_invalid,signer_key_source_production_managed_external_required,signer_quorum_minimum_not_met,signer_rotation_epoch_stale,signer_rotation_rehearsal_drift_detected,signer_rotation_promotion_stalled,fallback_signer_secret_present_violation,fallback_signer_secret_checkpoint_reason_mismatch,fallback_signer_secret_remediation_missing,quorum_evidence_missing,quorum_evidence_rotation_metadata_missing,quorum_evidence_rotation_metadata_invalid,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_drift_telemetry_missing,runtime_signer_drift_telemetry_rotation_delta_invalid,runtime_signer_drift_matrix_inputs_invalid,runtime_signer_drift_rotation_fail_threshold_exceeded,runtime_signer_drift_quorum_fail_threshold_exceeded,custody_continuity_bypass_detected",
    "rotation_preflight_reason_codes_value=none|<csv>",
    "custody_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-custody-reason-taxonomy.v1",
    "custody_reason_codes_csv=custody_evidence_missing,custody_evidence_sha256_invalid,custody_evidence_file_missing,quorum_evidence_custody_sha256_mismatch,custody_continuity_bypass_detected",
    "custody_reason_codes_value=none|<csv>",
    "quorum_evidence_approval_count_mismatch",
    "quorum_evidence_custody_sha256_mismatch",
    "custody_continuity_bypass_detected",
    "Regression: #4169",
    "Regression: #4170",
];

#[test]
fn checklist_contains_rotation_preflight_quorum_parity_and_custody_reason_mapping_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_ROTATION_PREFLIGHT_QUORUM_PARITY_AND_CUSTODY_REASON_MAPPING_GATE_MARKERS, "checklist_contains_rotation_preflight_quorum_parity_and_custody_reason_mapping_gate");
}

const CHECKLIST_CONTAINS_DEPLOYMENT_PREFLIGHT_MARKER_COMPLETENESS_SCHEMA_DRIFT_GATE_MARKERS: &[&str] = &[
    "## Deployment Preflight Marker Completeness and Schema Drift Rejection Gate (Issues #4146, #4150)",
    "test_check_local_kolme_live_deployment_preflight_policy.sh",
    "check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-deployment-preflight-policy.json",
    "deployment_preflight_marker_contract_status=verified",
    "deployment_preflight_marker_contract_version=kamn.kolme.local-live-deployment-preflight-marker-contract.v1",
    "deployment_preflight_required_markers_csv=rotation_preflight_reason_taxonomy_version,rotation_preflight_reason_codes_csv,rotation_preflight_reason_codes_value,custody_reason_taxonomy_version,custody_reason_codes_csv,custody_reason_codes_value",
    "deployment_preflight_schema_parity_status=verified",
    "deployment_preflight_schema_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-schema-parity-reason-taxonomy.v1",
    "deployment_preflight_schema_reason_codes_csv=deployment_preflight_required_marker_missing,deployment_preflight_schema_parity_mismatch,deployment_preflight_reason_taxonomy_version_mismatch,deployment_preflight_reason_codes_csv_mismatch,deployment_preflight_reason_codes_value_mismatch",
    "deployment_preflight_schema_reason_code=none|<reason>",
    "deployment_preflight_required_marker_missing:<marker>",
    "deployment_preflight_schema_parity_mismatch:<field>",
    "deployment_preflight_reason_taxonomy_version_mismatch",
    "Regression: #4146",
    "Regression: #4150",
];

#[test]
fn checklist_contains_deployment_preflight_marker_completeness_schema_drift_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_DEPLOYMENT_PREFLIGHT_MARKER_COMPLETENESS_SCHEMA_DRIFT_GATE_MARKERS, "checklist_contains_deployment_preflight_marker_completeness_schema_drift_gate");
}

const CHECKLIST_CONTAINS_DRY_RUN_WORKFLOW_MARKERS: &[&str] = &[
    "## Deterministic Dry-Run Workflow",
    "1. Create release candidate tag",
    "2. Rehearse migration on staging snapshot",
    "3. Execute bounded smoke and invariant suites",
    "4. Capture and sign dry-run evidence bundle",
];

#[test]
fn checklist_contains_dry_run_workflow() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_DRY_RUN_WORKFLOW_MARKERS, "checklist_contains_dry_run_workflow");
}
