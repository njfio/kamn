const ROADMAP: &str = include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");

#[test]
fn roadmap_tracks_runtime_decomposition_tranche4_snapshot_module_extraction() {
    assert!(ROADMAP.contains("Task #3090"));
    assert!(ROADMAP.contains("Task #3092"));
    assert!(ROADMAP.contains("Subtask #3093"));
    assert!(ROADMAP.contains("crates/kamn-core/src/runtime_snapshot_store.rs"));
    assert!(ROADMAP.contains("runtime_module_extraction_contract.rs"));
    assert!(ROADMAP.contains("runtime_network_docs.rs"));
}
