const DOC: &str = include_str!("../../../docs/architecture/data-layer-runtime-wiring.md");

#[test]
fn doc_contains_m0_m11_extraction_map_and_compatibility_markers() {
    assert!(DOC.contains("m0_m11_extraction_map_version=kamn.arch.data-layer-m0-m11-extraction-map.v1"));
    assert!(DOC.contains(
        "m0_m11_extraction_sequence_csv=M0,M1,M2,M3,M4,M5,M6,M7,M8,M9,M10,M11"
    ));
    assert!(DOC.contains(
        "m11_extraction_target_crate=crates/kamn-data-layer"
    ));
    assert!(DOC.contains(
        "m11_compatibility_shim_path=kamn-core::data_layer_m11_hardening_readiness"
    ));
    assert!(DOC.contains(
        "m11_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m11_hardening_readiness.rs,crates/kamn-core/tests/data_layer_m11_closure_evidence.rs,crates/kamn-data-layer/tests/data_layer_m11_hardening_readiness_integration.rs"
    ));
}
