use super::super::support::*;

#[test]
fn spec_c09_phase5a_docs_markers_present() {
    assert_doc_markers(
        "docs/research/e2e-live-testing-prd-phase5a-gap-analysis.md",
        "phase-5a docs marker artifact should exist",
        &[
        "phase5a_status_before=partial",
        "phase5a_process_runtime_contract=implemented",
        "phase5a_status_after=implemented",
        ],
    );
}

#[test]
fn spec_c10_milestone_index_references_active_phase5a_issue() {
    assert_milestone_markers(&[
        "#5584",
    ]);
}
