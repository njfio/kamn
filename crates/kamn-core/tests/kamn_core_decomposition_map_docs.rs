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

fn assert_contains_all(doc: &str, required_markers: &[&str]) {
    for marker in required_markers {
        assert!(doc.contains(marker), "missing required marker: {marker}");
    }
}

fn is_tranche_row(line: &str) -> bool {
    matches!(
        line.split('|').nth(1).map(str::trim),
        Some("T1" | "T2" | "T3" | "T4" | "T5" | "T6" | "T7")
    )
}

#[test]
fn module_map_declares_decomposition_tranche_markers() {
    assert_contains_all(
        MODULE_MAP_DOC,
        &[
            "## Decomposition Tranche Roadmap (Issue #6275)",
            "kamn_core_decomposition_map_version=kamn.arch.kamn-core-decomposition-map.v1",
            "kamn_core_decomposition_reason_taxonomy_version=kamn.arch.kamn-core-decomposition-reason-taxonomy.v1",
            "kamn_core_decomposition_reason_codes_csv=module_group_boundary_missing,tranche_ordering_missing,target_destination_missing,hotspot_prioritization_missing,architecture_index_link_missing",
            "kamn_core_decomposition_tranche_count=7",
            "kamn_core_decomposition_target_crates_csv=kamn-runtime-guards,kamn-snapshot-journal,kamn-types,kamn-data-layer,kamn-kolme,kamn-bridges,kamn-crypto",
        ],
    );
}

#[test]
fn module_map_lists_top_monolith_hotspots() {
    assert!(MODULE_MAP_DOC.contains("## Top Monolith Hotspots (By LOC)"));
    for (file, loc) in [
        ("message_lifecycle.rs", "1780"),
        ("channel_models.rs", "1780"),
        ("p2p_transport/p2p_transport_live.rs", "1711"),
        ("task_operations.rs", "1685"),
        ("did_registry.rs", "1679"),
    ] {
        assert!(MODULE_MAP_DOC.contains(&format!("| `{file}` | `{loc}` |")));
    }
}

#[test]
fn module_map_keeps_tranche_table_shape_stable() {
    let tranche_count = parse_usize_marker(MODULE_MAP_DOC, "kamn_core_decomposition_tranche_count");
    let table_rows = MODULE_MAP_DOC
        .lines()
        .filter(|line| is_tranche_row(line))
        .count();

    assert_eq!(table_rows, tranche_count, "tranche row count drifted");
}

#[test]
fn architecture_index_links_decomposition_anchor() {
    assert!(ARCHITECTURE_INDEX_DOC.contains(
        "docs/architecture/kamn-core-module-map.md#decomposition-tranche-roadmap-issue-6275"
    ));
}
