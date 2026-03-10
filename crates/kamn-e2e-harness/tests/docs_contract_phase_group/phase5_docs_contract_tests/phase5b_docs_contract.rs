use super::super::support::*;

#[test]
fn spec_c09_phase5b_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase5b-gap-analysis.md",
        "phase-5b docs marker artifact should exist",
        &[
        "phase5b_status_before=partial",
        "phase5b_process_lifecycle_contract=implemented",
        "phase5b_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c10_milestone_index_references_active_phase5b_issue() {
    assert_milestone_markers(&[
        "#5586",
    ]);
}
