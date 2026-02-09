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
fn doc_includes_cross_chain_receipt_finality_test_command() {
    assert!(DOC.contains("cargo test -p kamn-core --test cross_chain_receipt_finality"));
}
