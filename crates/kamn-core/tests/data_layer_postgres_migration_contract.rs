use std::fs;
use std::path::Path;

fn repo_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

fn load_required_file(relative: &str) -> String {
    let absolute = repo_path(relative);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|error| panic!("required contract artifact missing: {relative} ({error})"))
}

fn missing_markers<'a>(source: &'a str, required: &[&'a str]) -> Vec<&'a str> {
    required
        .iter()
        .copied()
        .filter(|marker| !source.contains(*marker))
        .collect()
}

#[test]
fn spec_c01_data_layer_baseline_migration_declares_required_tables() {
    let migration_relative = "migrations/202602190001_data_layer_phase1_bootstrap.sql";
    let migration_absolute = repo_path(migration_relative);
    assert!(
        Path::new(migration_absolute.as_str()).exists(),
        "baseline migration should exist at {migration_relative}"
    );

    let migration_source = load_required_file(migration_relative);
    let required_table_markers = [
        "CREATE TABLE IF NOT EXISTS messages",
        "CREATE TABLE IF NOT EXISTS merkle_batches",
        "CREATE TABLE IF NOT EXISTS did_registry",
        "CREATE TABLE IF NOT EXISTS escrows",
        "CREATE TABLE IF NOT EXISTS key_rotation_log",
        "CREATE TABLE IF NOT EXISTS access_log",
    ];
    let missing = missing_markers(&migration_source, &required_table_markers);
    assert!(
        missing.is_empty(),
        "baseline migration is missing required table markers: {missing:?}"
    );
}

#[test]
fn spec_c02_data_layer_baseline_migration_declares_index_and_rls_markers() {
    let migration_source =
        load_required_file("migrations/202602190001_data_layer_phase1_bootstrap.sql");
    let required_markers = [
        "CREATE INDEX IF NOT EXISTS idx_messages_owner_created_at",
        "CREATE INDEX IF NOT EXISTS idx_messages_non_shredded_created_at",
        "CREATE INDEX IF NOT EXISTS idx_messages_blind_indexes_gin",
        "-- KAMN_M2_RLS_MARKER:ENABLE_RLS_MESSAGES",
        "-- KAMN_M2_RLS_MARKER:MESSAGES_OWNER_SELECT_POLICY_TEMPLATE",
        "-- KAMN_M3_INDEX_MARKER:BLIND_INDEX_GIN_READY",
        "-- KAMN_M8_RETENTION_MARKER:SHREDDED_AT_PARTIAL_INDEX_READY",
    ];
    let missing = missing_markers(&migration_source, &required_markers);
    assert!(
        missing.is_empty(),
        "baseline migration is missing required index/RLS markers: {missing:?}"
    );
}

#[test]
fn spec_c03_marker_detection_fails_closed_when_required_markers_are_missing() {
    // Regression: #5255
    let source = "CREATE TABLE IF NOT EXISTS messages (...);";
    let required = [
        "CREATE TABLE IF NOT EXISTS messages",
        "-- KAMN_M2_RLS_MARKER:ENABLE_RLS_MESSAGES",
    ];
    let missing = missing_markers(source, &required);
    assert_eq!(
        missing,
        vec!["-- KAMN_M2_RLS_MARKER:ENABLE_RLS_MESSAGES"],
        "marker detector should deterministically report missing markers"
    );
}

#[test]
fn spec_c04_phase_one_docs_link_epic_story_and_task() {
    let roadmap = load_required_file("../../docs/review/data-layer-roadmap.md");
    let activation_plan = load_required_file(
        "../../docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md",
    );
    let milestone_index = load_required_file(
        "../../specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md",
    );

    for issue in ["#5247", "#5248", "#5255"] {
        assert!(
            roadmap.contains(issue),
            "data-layer roadmap should reference issue {issue}"
        );
        assert!(
            activation_plan.contains(issue),
            "activation plan should reference issue {issue}"
        );
        assert!(
            milestone_index.contains(issue),
            "milestone index should reference issue {issue}"
        );
    }
}
