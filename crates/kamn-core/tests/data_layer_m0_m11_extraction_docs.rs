const DOC: &str = include_str!("../../../docs/architecture/data-layer-runtime-wiring.md");

#[test]
fn doc_contains_m0_m11_extraction_map_and_compatibility_markers() {
    let required_markers = [
        "m0_m11_extraction_map_version=kamn.arch.data-layer-m0-m11-extraction-map.v1",
        "m0_m11_extraction_sequence_csv=M0,M1,M2,M3,M4,M5,M6,M7,M8,M9,M10,M11",
        "m11_extraction_target_crate=crates/kamn-data-layer",
        "m11_compatibility_shim_path=kamn-core::data_layer_m11_hardening_readiness",
        "m11_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m11_hardening_readiness.rs,crates/kamn-core/tests/data_layer_m11_closure_evidence.rs,crates/kamn-data-layer/tests/data_layer_m11_hardening_readiness_integration.rs",
    ];
    for marker in required_markers {
        assert!(DOC.contains(marker), "missing marker: {marker}");
    }
}
