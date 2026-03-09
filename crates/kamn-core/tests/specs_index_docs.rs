const DOC: &str = include_str!("../../../specs/INDEX.md");
const REQUIRED_MARKERS: [&str; 7] = [
    "specs_index_version=kamn.docs.specs-index.v2",
    "specs_index_purpose=full top-level issue spec coverage and workflow orientation",
    "specs_index_scope=top_level_issue_specs_only",
    "specs_index_naming_pattern=specs/{issue}-{slug}.md",
    "specs_index_status_taxonomy_csv=planned,active,completed,superseded",
    "specs_index_shards_csv=specs/index/6000-6499.md,specs/index/6500-6999.md",
    "specs_index_coverage_authority=scripts/ci/check_specs_index_coverage.sh",
];

#[test]
fn doc_contains_specs_index_required_markers() {
    for marker in REQUIRED_MARKERS {
        assert!(
            DOC.contains(marker),
            "specs index should contain marker: {marker}"
        );
    }
}
