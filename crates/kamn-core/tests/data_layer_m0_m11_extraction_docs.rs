const DOC: &str = include_str!("../../../docs/architecture/data-layer-runtime-wiring.md");

#[test]
fn doc_contains_m0_m11_extraction_map_and_compatibility_markers() {
    let required_markers = [
        "m0_m11_extraction_map_version=kamn.arch.data-layer-m0-m11-extraction-map.v1",
        "m0_m11_extraction_sequence_csv=M0,M1,M2,M3,M4,M5,M6,M7,M8,M9,M10,M11",
        "m11_extraction_target_crate=crates/kamn-data-layer",
        "m11_compatibility_shim_path=kamn-core::data_layer_m11_hardening_readiness",
        "m11_contract_protection_tests_csv=crates/kamn-core/tests/data_layer_m11_hardening_readiness.rs,crates/kamn-core/tests/data_layer_m11_closure_evidence.rs,crates/kamn-data-layer/tests/data_layer_m11_hardening_readiness_integration.rs",
        "m10_retry_extraction_slice_version=kamn.arch.data-layer-m10-retry-extraction.v1",
        "m10_retry_extraction_target_crate=crates/kamn-data-layer",
        "m10_retry_compatibility_wrapper_path=kamn-core::data_layer_m10_partition_archival::retry",
        "m10_full_extraction_blocker_csv=data_layer_m8_compliance_lifecycle,KamnDid",
        "m10_projection_port_seam_version=kamn.arch.data-layer-m10-projection-port.v1",
        "m10_projection_port_trait_path=kamn-data-layer::DataLayerM10ComplianceProjectionPort",
        "m10_projection_port_entrypoint=DataLayerM10PartitionLifecycleRegistry::project_partition_shred_completeness_with_port",
        "m10_phase6_port_seam_version=kamn.arch.data-layer-m10-phase6-port.v1",
        "m10_phase6_port_trait_path=kamn-data-layer::DataLayerM10Phase6CompliancePort",
        "m10_phase6_orchestration_port_entrypoint=data_layer_m10_execute_phase6_orchestration_tick_with_port",
        "m10_phase6_scheduler_port_entrypoint=data_layer_m10_execute_phase6_scheduler_cycle_with_port",
        "m10_phase6_policy_extraction_slice_version=kamn.arch.data-layer-m10-phase6-policy-extraction.v1",
        "m10_phase6_policy_target_crate=crates/kamn-data-layer",
        "m10_phase6_policy_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6",
        "m10_phase6_scheduler_preflight_extraction_slice_version=kamn.arch.data-layer-m10-phase6-scheduler-preflight-extraction.v1",
        "m10_phase6_scheduler_preflight_target_crate=crates/kamn-data-layer",
        "m10_phase6_scheduler_preflight_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6",
        "m10_phase6_runtime_evidence_extraction_slice_version=kamn.arch.data-layer-m10-phase6-runtime-evidence-extraction.v1",
        "m10_phase6_runtime_evidence_target_crate=crates/kamn-data-layer",
        "m10_phase6_runtime_evidence_wrapper_path=kamn-core::data_layer_m10_partition_archival::phase6",
    ];
    for marker in required_markers {
        assert!(DOC.contains(marker), "missing marker: {marker}");
    }
}
