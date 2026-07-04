const TARGET_GRAPH_DOC: &str =
    include_str!("../../../docs/architecture/kamn-core-target-crate-graph.md");
const ARCHITECTURE_INDEX_DOC: &str = include_str!("../../../docs/architecture/README.md");
const KAMN_TYPES_DOC: &str = include_str!("../../../docs/architecture/kamn-types.md");

#[test]
fn target_graph_doc_declares_required_markers() {
    for marker in [
        "## Target Crate Graph (Issue #6647)",
        "kamn_core_target_crate_graph_version=kamn.arch.kamn-core-target-crate-graph.v1",
        "kamn_core_target_crate_graph_status=planned",
        "kamn_core_target_foundational_crates_csv=kamn-types,kamn-crypto,kamn-runtime-guards,kamn-snapshot-journal,kamn-bridges,kamn-data-layer,kamn-kolme,kamn-live-probe-matrix",
        "kamn_core_target_domain_crates_csv=kamn-governance,kamn-escrow,kamn-compliance",
        "kamn_core_target_forbidden_edges_csv=kamn-types->kamn-core,domain-crates->kamn-core-through-shims",
        "kamn_core_target_bridge_rule_csv=kamn-core-reexports-temporary,extracted-crates-must-not-depend-on-kamn-core",
        "kamn_core_target_migration_order_csv=types-inversion,governance,escrow,compliance",
        "kamn_core_target_module_map_source=docs/architecture/kamn-core-module-map.md",
    ] {
        assert!(
            TARGET_GRAPH_DOC.contains(marker),
            "missing target crate graph marker: {marker}"
        );
    }
}

#[test]
fn target_graph_doc_maps_candidate_module_groups() {
    for marker in [
        "`did`, `agent_key_hierarchy`, `key_lifecycle`, `key_recovery`",
        "`governance_workflow`, `operator_actions`, `operator_dashboard_api`, `operator_dashboard_ui`",
        "`task_operations`, `task_lifecycle`, `task_payment`, `task_artifacts`, `escrow`, `service_marketplace`, `token`",
        "`content_storage`, `content_retrieval`, `content_lifecycle`, `content_replication`, `data_classification`, `redaction_compliance`, `audit_exports`",
    ] {
        assert!(
            TARGET_GRAPH_DOC.contains(marker),
            "missing candidate module mapping marker: {marker}"
        );
    }
}

#[test]
fn architecture_index_links_target_graph_doc() {
    assert!(ARCHITECTURE_INDEX_DOC.contains("docs/architecture/kamn-core-target-crate-graph.md"));
}

#[test]
fn kamn_types_doc_declares_inversion_target() {
    for marker in [
        "kamn_types_current_dependency_status=owned-did-surface",
        "kamn_types_target_dependency_policy=no-kamn-core",
        "kamn_types_inversion_first_wave_csv=AgentDid,KamnDid,DidDocument,DidService,DidVerificationMethod",
    ] {
        assert!(
            KAMN_TYPES_DOC.contains(marker),
            "missing kamn-types inversion marker: {marker}"
        );
    }
}
