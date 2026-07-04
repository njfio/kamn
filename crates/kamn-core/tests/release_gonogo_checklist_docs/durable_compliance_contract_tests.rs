use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_DSAR_LEGAL_HOLD_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## DSAR Legal-Hold Evidence Contract",
    "generate_dsar_legal_hold_evidence_bundle.sh",
    "check_dsar_legal_hold_policy.sh",
    "run_dsar_legal_hold_contract_lane.sh",
    "run_dsar_legal_hold_deep_lane.sh",
    "run_dsar_legal_hold_matrix.py",
    "fixtures/compliance_dsar/legal_hold_precedence_cases.json",
];

#[test]
fn checklist_contains_dsar_legal_hold_evidence_contract() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_DSAR_LEGAL_HOLD_EVIDENCE_CONTRACT_MARKERS,
        "checklist_contains_dsar_legal_hold_evidence_contract",
    );
}

const CHECKLIST_CONTAINS_FEDERATED_DID_HANDSHAKE_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Federated DID Handshake Evidence Contract",
    "federated_did_handshake_contract.py",
    "generate_federated_did_handshake_evidence_bundle.sh",
    "check_federated_did_handshake_policy.sh",
    "run_federated_did_handshake_contract_lane.sh",
    "run_federated_did_handshake_deep_lane.sh",
    "run_federated_did_handshake_matrix.py",
    "check_federated_did_handshake_deep_policy.sh",
    "federated_did_handshake_deep_policy_contract.py",
    "run_federated_did_handshake_deep_policy_matrix.py",
    "fixtures/federated_did_handshake/partition_replay_cases.json",
    "cargo test -p kamn-core --test federated_did_handshake_runtime",
];

#[test]
fn checklist_contains_federated_did_handshake_evidence_contract() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_FEDERATED_DID_HANDSHAKE_EVIDENCE_CONTRACT_MARKERS,
        "checklist_contains_federated_did_handshake_evidence_contract",
    );
}

const CHECKLIST_CONTAINS_FEDERATED_DELEGATION_SETTLEMENT_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Federated Delegation Settlement Evidence Contract",
    "generate_federated_delegation_settlement_evidence_bundle.sh",
    "check_federated_delegation_settlement_policy.sh",
    "run_federated_delegation_settlement_contract_lane.sh",
    "run_federated_delegation_settlement_deep_lane.sh",
    "run_federated_delegation_settlement_matrix.py",
    "fixtures/federated_task_delegation/partition_replay_cases.json",
];

#[test]
fn checklist_contains_federated_delegation_settlement_evidence_contract() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_FEDERATED_DELEGATION_SETTLEMENT_EVIDENCE_CONTRACT_MARKERS,
        "checklist_contains_federated_delegation_settlement_evidence_contract",
    );
}

const CHECKLIST_CONTAINS_KOLME_VERSION_COMPATIBILITY_REPLAY_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Kolme Version Compatibility Replay Evidence Contract",
    "validate_version_compatibility.py",
    "generate_fork_compatibility_evidence.py",
    "check_fork_compatibility_policy.py",
    "check_upgrade_compatibility_marker_matrix_policy.py",
    "run_version_compatibility_replay.py",
    "check_runtime_commit_replay_policy.py",
    "run_runtime_commit_replay_tamper_matrix.py",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_runtime_commit_adapter_contract_lane.json --phase contract",
    "cargo test -p kamn-kolme --test runtime_commit_module_boundary_contracts",
    "cargo test -p kamn-core --test kolme_runtime_commit_import_boundary",
    "receipt_provider_mismatch",
    "receipt_not_final",
    "run_version_compatibility_contract_lane.sh",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_runtime_commit_replay_contract_lane.json --phase contract",
    "run_version_compatibility_replay_deep_lane.sh",
    "fixtures/kolme_compatibility/version_compatibility_cases.json",
    "fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json",
    "provider_failure_reason_taxonomy_version=kamn.kolme.local-runtime-commit-provider-failure-reason-taxonomy.v1",
    "reason_taxonomy_version=kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1",
    "reason_codes_csv=version_report_missing,fork_policy_report_missing,version_report_schema_mismatch,version_report_reason_taxonomy_mismatch,version_report_reason_codes_csv_mismatch,version_report_rehearsal_bypass_guard_status_mismatch,version_report_rehearsal_output_normalization_status_mismatch,fork_policy_report_schema_mismatch,fork_policy_report_reason_taxonomy_mismatch,fork_policy_report_reason_codes_csv_mismatch,fork_policy_report_rehearsal_bypass_guard_status_mismatch,fork_policy_report_rehearsal_output_normalization_status_mismatch,expected_final_decision_mismatch,ci_fast_gate_failed",
    "upgrade_compatibility_runbook_marker_parity_status=verified",
    "upgrade_compatibility_runbook_reason_taxonomy_version=kamn.kolme.upgrade-compatibility-runbook-reason-taxonomy.v1",
    "upgrade_compatibility_runbook_reason_codes_csv=upgrade_compatibility_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "upgrade_compatibility_runbook_reason_code=none|<reason>",
    "upgrade_compatibility_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "version_report_schema_mismatch",
    "fork_policy_report_rehearsal_bypass_guard_status_mismatch",
    "expected_final_decision_mismatch",
    "ci_fast_gate_failed",
    "provider_failure_reason_codes_csv=provider_client_contract_mismatch,provider_contract_enforcement_mode_mismatch,provider_live_contract_marker_mismatch,provider_live_contract_marker_missing,provider_in_memory_reference_detected,provider_hint_in_memory_provider_reference_detected,provider_submit_profile_contract_mismatch,provider_command_marker_mismatch,provider_command_marker_missing,provider_signing_profile_marker_mismatch,provider_signing_profile_marker_missing,provider_signing_profile_simulated_detected,provider_signer_adapter_contract_mismatch,provider_signing_curve_contract_mismatch,provider_signing_profile_contract_version_mismatch,live_command_in_memory_provider_reference_detected",
    "request_payload_evidence_artifact_path_lineage_mismatch",
    "submit_evidence_artifact_path_lineage_mismatch",
    "finality_evidence_artifact_path_lineage_mismatch",
    "runtime_signing_profile_contract_version=v1",
    "runtime_signing_profile=kolme-fork-secp256k1-v1",
    "native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1",
    "native_signer_reason_codes_csv=runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch",
    "Regression: #4372",
    "Regression: #4373",
    "Regression: #4378",
    "Regression: #4380",
    "Regression: #4180",
    "Regression: #4181",
    "Regression: #4182",
    "Regression: #4183",
];

#[test]
fn checklist_contains_kolme_version_compatibility_replay_evidence_contract() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_KOLME_VERSION_COMPATIBILITY_REPLAY_EVIDENCE_CONTRACT_MARKERS,
        "checklist_contains_kolme_version_compatibility_replay_evidence_contract",
    );
}
