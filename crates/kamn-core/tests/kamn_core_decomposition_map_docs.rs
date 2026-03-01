const MODULE_MAP_DOC: &str = include_str!("../../../docs/architecture/kamn-core-module-map.md");
const ARCHITECTURE_INDEX_DOC: &str = include_str!("../../../docs/architecture/README.md");

fn marker_value<'a>(doc: &'a str, key: &str) -> Option<&'a str> {
    doc.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .map(str::trim)
}

fn parse_usize_marker(doc: &str, key: &str) -> usize {
    marker_value(doc, key)
        .unwrap_or_else(|| panic!("missing marker: {key}"))
        .parse::<usize>()
        .unwrap_or_else(|error| panic!("invalid usize marker {key}: {error}"))
}

#[test]
fn module_map_declares_decomposition_tranche_markers() {
    assert!(MODULE_MAP_DOC.contains("## Decomposition Tranche Roadmap (Issue #6275)"));
    assert!(MODULE_MAP_DOC
        .contains("kamn_core_decomposition_map_version=kamn.arch.kamn-core-decomposition-map.v1"));
    assert!(
        MODULE_MAP_DOC.contains("kamn_core_decomposition_reason_taxonomy_version=kamn.arch.kamn-core-decomposition-reason-taxonomy.v1")
    );
    assert!(MODULE_MAP_DOC.contains("kamn_core_decomposition_reason_codes_csv=module_group_boundary_missing,tranche_ordering_missing,target_destination_missing,hotspot_prioritization_missing,architecture_index_link_missing"));
    assert!(MODULE_MAP_DOC.contains("kamn_core_decomposition_tranche_count=5"));
    assert!(
        MODULE_MAP_DOC.contains("kamn_core_decomposition_target_crates_csv=kamn-runtime-guards,kamn-snapshot-journal,kamn-kolme,kamn-bridges,kamn-crypto")
    );
}

#[test]
fn module_map_lists_top_monolith_hotspots() {
    assert!(MODULE_MAP_DOC.contains("## Top Monolith Hotspots (By LOC)"));
    assert!(MODULE_MAP_DOC.contains("| `message_lifecycle.rs` | `1780` |"));
    assert!(MODULE_MAP_DOC.contains("| `channel_models.rs` | `1780` |"));
    assert!(MODULE_MAP_DOC.contains("| `p2p_transport/p2p_transport_live.rs` | `1711` |"));
    assert!(MODULE_MAP_DOC.contains("| `task_operations.rs` | `1685` |"));
    assert!(MODULE_MAP_DOC.contains("| `did_registry.rs` | `1679` |"));
}

#[test]
fn module_map_keeps_tranche_table_shape_stable() {
    let tranche_count = parse_usize_marker(MODULE_MAP_DOC, "kamn_core_decomposition_tranche_count");
    let table_rows = MODULE_MAP_DOC
        .lines()
        .filter(|line| {
            line.starts_with("| T1 ")
                || line.starts_with("| T2 ")
                || line.starts_with("| T3 ")
                || line.starts_with("| T4 ")
                || line.starts_with("| T5 ")
        })
        .count();

    assert_eq!(table_rows, tranche_count, "tranche row count drifted");
}

#[test]
fn architecture_index_links_decomposition_anchor() {
    assert!(ARCHITECTURE_INDEX_DOC
        .contains("docs/architecture/kamn-core-module-map.md#decomposition-tranche-roadmap-issue-6275"));
}
