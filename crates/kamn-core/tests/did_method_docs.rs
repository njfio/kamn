const DID_METHOD_DOC: &str = include_str!("../../../docs/foundation/did-method.md");

#[test]
fn did_method_doc_contains_core_rules() {
    assert!(DID_METHOD_DOC.contains("## DID Validation Rules"));
    assert!(DID_METHOD_DOC.contains("DID must start with `kamn:did:agent:`."));
    assert!(DID_METHOD_DOC.contains("## Canonical DID Document Rules"));
}

#[test]
fn did_method_doc_contains_federated_handshake_contract() {
    assert!(DID_METHOD_DOC.contains("## Federated DID Handshake Evidence Contract"));
    assert!(DID_METHOD_DOC.contains("generate_federated_did_handshake_evidence_bundle.sh"));
    assert!(DID_METHOD_DOC.contains("check_federated_did_handshake_policy.sh"));
    assert!(DID_METHOD_DOC.contains("run_federated_did_handshake_contract_lane.sh"));
    assert!(DID_METHOD_DOC.contains("run_federated_did_handshake_deep_lane.sh"));
    assert!(DID_METHOD_DOC.contains("run_federated_did_handshake_matrix.py"));
    assert!(DID_METHOD_DOC.contains("fixtures/federated_did_handshake/partition_replay_cases.json"));
}

#[test]
fn regression_requires_federated_handshake_replay_guard_marker() {
    // Regression: #734
    assert!(DID_METHOD_DOC
        .contains("replay/downgrade/tamper attempts force `NO-GO` (`Regression: #734`)."));
}
