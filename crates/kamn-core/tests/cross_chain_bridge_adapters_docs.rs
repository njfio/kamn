#[test]
fn cross_chain_bridge_adapters_docs_migrated_to_wave3_harness() {
    let harness = std::fs::read_to_string("tests/docs_contract_wave3_harness.rs")
        .expect("docs contract wave3 harness must exist");
    assert!(harness.contains("mod cross_chain_bridge_adapters_docs {"));
}
