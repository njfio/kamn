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
}

#[test]
fn matrix_contains_quorum_attestation_replay_guard_entry_details() {
    assert!(
        CONTROL_MATRIX.contains("Quorum attestation evidence drift or replayed approval artifact")
    );
}
