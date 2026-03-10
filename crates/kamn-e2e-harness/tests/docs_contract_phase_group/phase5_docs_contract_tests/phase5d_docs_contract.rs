use super::super::support::*;

#[test]
fn spec_c07_phase5d_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase5d-gap-analysis.md",
        "phase-5d docs marker artifact should exist",
        &[
        "phase5d_status_before=partial",
        "phase5d_live_validation_contract=implemented",
        "phase5d_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c08_milestone_index_references_active_phase5d_issue() {
    assert_milestone_markers(&[
        "#5590",
    ]);
}
