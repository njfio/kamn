const PROFILE: &str = include_str!("../../../docs/foundation/didcomm-v2-compatibility-profile.md");

#[test]
fn profile_contains_field_level_mapping_table() {
    assert!(PROFILE.contains("## Field-Level Mapping"));
    assert!(PROFILE.contains("| KAMN Canonical Field | DIDComm v2 Field | Compatibility Rule |"));
    assert!(PROFILE.contains("| envelope.id | id | Preserve as-is. |"));
    assert!(PROFILE.contains("| envelope.from | from | Preserve as DID string. |"));
    assert!(PROFILE.contains("| envelope.to[] | to[] | Preserve recipient DID list order. |"));
    assert!(PROFILE.contains("| body.message | body | Serialize as JSON body payload. |"));
}

#[test]
fn profile_contains_crypto_and_key_handling_expectations() {
    assert!(PROFILE.contains("## Crypto and Key Handling Expectations"));
    assert!(PROFILE
        .contains("Ed25519 verification methods remain authoritative for signature validation."));
    assert!(PROFILE.contains("X25519 key agreement references must map to recipient key IDs."));
    assert!(
        PROFILE.contains("Unsupported algorithm negotiation results in compatibility rejection.")
    );
}

#[test]
fn profile_contains_deterministic_test_vectors() {
    assert!(PROFILE.contains("## Deterministic Compatibility Vectors"));
    assert!(PROFILE
        .contains("Vector-S1: canonical request envelope maps to DIDComm plaintext message."));
    assert!(
        PROFILE.contains("Vector-S2: canonical response envelope maps to DIDComm signed response.")
    );
    assert!(PROFILE.contains("Vector-F1: missing recipient key reference is rejected."));
    assert!(PROFILE.contains("Vector-F2: unsupported attachment mapping is rejected."));
}

#[test]
fn regression_requires_unsupported_attachment_rejection_rule() {
    // Regression: #179
    assert!(PROFILE.contains("Unsupported attachment translation decision: reject."));
}

#[test]
fn profile_contains_didcomm_envelope_replay_contract_lane_commands() {
    assert!(PROFILE.contains("## DIDComm Envelope Compatibility Replay Contract Lane (Issue #892)"));
    assert!(PROFILE.contains("run_didcomm_envelope_compatibility_replay.py"));
    assert!(PROFILE.contains("check_didcomm_envelope_compatibility_policy.sh"));
    assert!(PROFILE.contains("run_didcomm_envelope_compatibility_contract_lane.sh"));
    assert!(PROFILE.contains("didcomm_envelope_compatibility_reason_codes:GO:v1"));
}

#[test]
fn regression_requires_didcomm_envelope_schema_signature_drift_fail_closed_policy() {
    // Regression: #892
    assert!(PROFILE.contains("Regression: #892"));
}
