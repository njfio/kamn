const DID_METHOD_DOC: &str = include_str!("../../../docs/foundation/did-method.md");

#[test]
fn did_method_doc_contains_core_rules() {
    assert!(DID_METHOD_DOC.contains("## DID Validation Rules"));
    assert!(DID_METHOD_DOC.contains("DID must start with `kamn:did:agent:`."));
    assert!(DID_METHOD_DOC.contains("## Canonical DID Document Rules"));
    assert!(DID_METHOD_DOC.contains(
        "Canonical service endpoint policy enforces `kamn://messaging/<method-specific-id>`."
    ));
    assert!(DID_METHOD_DOC.contains(
        "Service endpoint canonicalization normalizes scheme/authority/identifier to lowercase."
    ));
    assert!(DID_METHOD_DOC
        .contains("Service endpoints with query/fragment or multi-segment paths are rejected."));
}

#[test]
fn did_method_doc_contains_federated_handshake_contract() {
    assert!(DID_METHOD_DOC.contains("## Federated DID Handshake Evidence Contract"));
    assert!(DID_METHOD_DOC.contains("federated_did_handshake_contract.py"));
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

#[test]
fn regression_requires_service_endpoint_canonicalization_guard_marker() {
    // Regression: #1000
    assert!(DID_METHOD_DOC.contains(
        "non-canonical service endpoint scheme/authority/path combinations must remain rejected (`Regression: #1000`)."
    ));
}
