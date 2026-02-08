const DOC: &str = include_str!("../../../docs/foundation/typescript-sdk-beta.md");

#[test]
fn doc_contains_schema_and_sdk_scope() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("packages/kamn-schema"));
    assert!(DOC.contains("packages/kamn-sdk"));
    assert!(DOC.contains("validateCanonicalMessageEnvelope(...)"));
    assert!(DOC.contains("KAMNClient"));
    assert!(DOC.contains("LiveTransportKAMNClient"));
    assert!(DOC.contains("TransportModeMismatchError"));
}

#[test]
fn doc_contains_fast_validation_strategy() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("node --experimental-strip-types --test"));
    assert!(DOC.contains("bash scripts/sdk/run_live_transport_parity_contract_lane.sh"));
    assert!(DOC.contains("npm --prefix packages/kamn-schema test"));
    assert!(DOC.contains("npm --prefix packages/kamn-sdk test"));
}

#[test]
fn regression_requires_nonce_rule_and_proof_binding_rule() {
    // Regression: #218
    assert!(DOC.contains("nonce must be a positive integer."));
    assert!(DOC.contains("proof verification method must be bound to sender DID"));
    assert!(DOC.contains("`TransportModeMismatchError` (`Regression: #620`)"));
}
