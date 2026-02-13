const DOC: &str = include_str!("../../../docs/ci/strategy.md");

#[test]
fn ci_strategy_doc_contains_live_https_feature_gate_contract() {
    assert!(DOC.contains("## Kolme Live HTTPS Feature-Gate Contract"));
    assert!(DOC.contains("`live-https` feature remains enabled by default"));
    assert!(DOC.contains("cargo check -p kamn-core --features live-https"));
    assert!(DOC.contains("cargo check -p kamn-core --no-default-features"));
}
