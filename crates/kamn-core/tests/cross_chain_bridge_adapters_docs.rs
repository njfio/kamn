const DOC: &str = include_str!("../../../docs/foundation/cross-chain-bridge-adapters.md");

#[test]
fn doc_covers_adapter_and_receipt_normalization_scopes() {
    assert!(DOC.contains("# Cross-Chain Bridge Adapters"));
    assert!(DOC.contains("CrossChainBridgeEngine"));
    assert!(DOC.contains("normalize_cross_chain_receipt(...)"));
}

#[test]
fn doc_lists_ethereum_and_near_finality_rules() {
    assert!(DOC.contains("## Receipt Finality Normalization Rules (Ethereum / Near)"));
    assert!(DOC.contains("finalized` or `safe` with at least `12` confirmations"));
    assert!(DOC.contains("`final` -> `Final`"));
}

#[test]
fn doc_contains_outbound_intent_attestation_and_idempotency_rules() {
    assert!(DOC.contains("## Outbound Intent Attestation and Retry Idempotency Rules"));
    assert!(DOC.contains("idempotency key must be `idemp:<value>`"));
    assert!(DOC.contains("duplicate request flag forces deterministic `NO-GO`"));
    assert!(DOC.contains("Regression: #742"));
}

#[test]
fn doc_includes_cross_chain_receipt_finality_test_command() {
    assert!(DOC.contains("cargo test -p kamn-core --test cross_chain_receipt_finality"));
    assert!(DOC.contains("test_generate_cross_chain_outbound_intent_evidence_bundle.sh"));
    assert!(DOC.contains("test_run_cross_chain_outbound_intent_contract_lane.sh"));
}
