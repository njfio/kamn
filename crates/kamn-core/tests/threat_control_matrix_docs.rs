const CONTROL_MATRIX: &str = include_str!("../../../docs/foundation/threat-control-matrix.md");

#[test]
fn matrix_contains_required_columns() {
    assert!(CONTROL_MATRIX.contains(
        "| Threat ID | Threat | Control | Enforcement Point | Owner | Validation Test |"
    ));
}

#[test]
fn matrix_contains_core_threat_entries() {
    assert!(CONTROL_MATRIX.contains("TM-001"));
    assert!(CONTROL_MATRIX.contains("TM-002"));
    assert!(CONTROL_MATRIX.contains("TM-003"));
    assert!(CONTROL_MATRIX.contains("TM-004"));
    assert!(CONTROL_MATRIX.contains("TM-005"));
    assert!(CONTROL_MATRIX.contains("TM-006"));
    assert!(CONTROL_MATRIX.contains("TM-007"));
    assert!(CONTROL_MATRIX.contains("TM-008"));
    assert!(CONTROL_MATRIX.contains("TM-009"));
}

#[test]
fn matrix_maps_controls_to_tests() {
    assert!(CONTROL_MATRIX.contains("verify_instruction_signature_path"));
    assert!(CONTROL_MATRIX.contains("reject_out_of_sequence_nonce_per_sender"));
    assert!(CONTROL_MATRIX.contains("escrow_lifecycle_illegal_transition_rejected"));
    assert!(CONTROL_MATRIX.contains(
        "integration_signature_profile_fixture_matrix_remains_consistent_with_transaction_guards"
    ));
    assert!(CONTROL_MATRIX.contains("quorum_attestation_replay_guard_policy_contract"));
    assert!(CONTROL_MATRIX.contains("governance_quorum_attestation_replay_policy_contract.py"));
    assert!(CONTROL_MATRIX.contains("governance_quorum_attestation_replay_lane_contract.py"));
    assert!(CONTROL_MATRIX.contains("run_quorum_attestation_replay_contract_lane.sh"));
    assert!(CONTROL_MATRIX.contains("run_signer_policy_contract_lane.sh"));
    assert!(CONTROL_MATRIX
        .contains("functional_privileged_roles_deny_fallback_when_provider_unavailable"));
    assert!(CONTROL_MATRIX
        .contains("router_decision_matrix_distinguishes_unavailable_vs_policy_blocked_handshakes"));
    assert!(CONTROL_MATRIX
        .contains("regression_provider_client_backend_mismatch_is_rejected_without_fallback"));
    assert!(CONTROL_MATRIX.contains("run_watchdog_proof_consensus_contract_lane.sh"));
    assert!(CONTROL_MATRIX.contains("run_watchdog_proof_consensus_deep_lane.sh"));
    assert!(CONTROL_MATRIX.contains("check_watchdog_proof_consensus_policy.sh"));
    assert!(CONTROL_MATRIX.contains("watchdog_proof_consensus_reason_codes:GO:v1"));
    assert!(CONTROL_MATRIX.contains("watchdog_proof_consensus_reason_codes:NO-GO:v1"));
    assert!(CONTROL_MATRIX.contains("run_signature_parity_contract_lane.sh"));
    assert!(CONTROL_MATRIX.contains("run_signature_parity_matrix.py"));
    assert!(CONTROL_MATRIX.contains("check_signature_parity_policy.py"));
    assert!(CONTROL_MATRIX.contains("parity_signature_mismatch"));
    assert!(CONTROL_MATRIX.contains("parity_recovery_id_mismatch"));
    assert!(CONTROL_MATRIX.contains("parity_pubkey_mismatch"));
}

#[test]
fn matrix_contains_quorum_attestation_replay_guard_entry_details() {
    assert!(
        CONTROL_MATRIX.contains("Quorum attestation evidence drift or replayed approval artifact")
    );
}

#[test]
fn matrix_contains_signer_fallback_policy_entry_details() {
    assert!(CONTROL_MATRIX
        .contains("Privileged role fallback bypass under secure-provider degradation"));
    assert!(CONTROL_MATRIX.contains("`Regression: #987`"));
}

#[test]
fn matrix_contains_watchdog_proof_consensus_entry_details() {
    assert!(CONTROL_MATRIX.contains(
        "Validator/watchdog proof-consensus anomaly evidence missing or cadence/budget guard bypass"
    ));
    assert!(CONTROL_MATRIX.contains("`Regression: #996`"));
}

#[test]
fn matrix_contains_signature_parity_entry_details() {
    assert!(CONTROL_MATRIX
        .contains("Kolme live signature conformance drift or malformed parity evidence"));
    assert!(CONTROL_MATRIX.contains("`Regression: #2299`"));
}
