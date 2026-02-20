#[test]
fn docs_contract_template_guidance_contract_migrated_to_wave3_harness() {
    let harness = std::fs::read_to_string("tests/docs_contract_wave3_harness.rs")
        .expect("docs contract wave3 harness must exist");
    assert!(harness.contains("mod docs_contract_template_guidance_contract {"));
}
