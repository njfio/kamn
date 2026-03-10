use super::super::support::*;

#[test]
fn spec_c14_phase6c_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase6c-gap-analysis.md",
        "phase-6c docs marker artifact should exist",
        &[
        "phase6c_status_before=partial",
        "phase6c_live_process_execution_contract=implemented",
        "phase6c_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c15_milestone_index_references_active_phase6c_issue() {
    assert_milestone_markers(&[
        "#5596",
    ]);
}
