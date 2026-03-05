const DOC: &str = include_str!("../../../specs/INDEX.md");
const REQUIRED_MARKERS: [&str; 5] = [
    "specs_index_version=kamn.docs.specs-index.v1",
    "specs_index_purpose=spec navigation and workflow orientation",
    "specs_index_naming_pattern=specs/{issue}-{slug}.md",
    "specs_index_status_taxonomy_csv=planned,active,completed,superseded",
    "specs_index_curated_tracks_csv=m10_phase6_extraction,cli_contract_followups,security_runtime_hardening",
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
