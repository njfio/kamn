use super::super::support::*;

#[test]
fn spec_c08_phase5c_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase5c-gap-analysis.md",
        "phase-5c docs marker artifact should exist",
        &[
        "phase5c_status_before=partial",
        "phase5c_spawn_timeline_contract=implemented",
        "phase5c_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c09_milestone_index_references_active_phase5c_issue() {
    assert_milestone_markers(&[
        "#5588",
    ]);
}
