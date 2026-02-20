#[test]
fn message_delivery_guards_docs_migrated_to_wave3_harness() {
    let harness = std::fs::read_to_string("tests/docs_contract_wave3_harness.rs")
        .expect("docs contract wave3 harness must exist");
    assert!(harness.contains("mod message_delivery_guards_docs {"));
}
